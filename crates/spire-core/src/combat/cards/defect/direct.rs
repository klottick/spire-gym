//! M20 direct-card producers. Shared mutation/history/queue ownership stays above.
use super::*;
use super::super::model::OwnedCard;
use crate::content::{CardCost, CardId, CardTarget, Catalog};

#[derive(Clone,Debug,PartialEq,Eq)]
pub enum DirectEffect {
    Aggregate { divisor:u8 },
    DoubleEnergy,
    RandomHit { base:i32, remaining:u32 },
    Chaos { requested:u8, orbs:Vec<OrbKind> },
    Melter { damage:i32 },
    Sunder { damage:i32 },
    SunderCheck,
}

fn is_direct(id:CardId)->bool {
    use CardId::*;
    matches!(id,BeamCell|ChargeBattery|GoForTheEyes|Leap|Stack|SweepingBeam|Turbo|Aggregate
        |AutoShields|Blizzard|BootSequence|Chaos|Chill|DoomAndGloom|DoubleEnergy|Glacier
        |Melter|Overclock|Reprogram|RipAndTear|Skim|Sunder|CoreSurge|Hyperbeam
        |MeteorStrike|Rainbow|ThunderStrike)
}

fn is_legacy(id:CardId)->bool {
    matches!(id,CardId::BeamCell|CardId::GoForTheEyes|CardId::Leap|CardId::SweepingBeam)
}

fn printed_cost(cost:CardCost)->i32 {
    match cost { CardCost::Energy(amount)=>i32::from(amount),CardCost::X=>0 }
}

fn run_error(error:RunError)->InputError {
    match error {
        RunError::InvalidAction=>InputError::Invariant("captured direct-card target or value became invalid".into()),
        RunError::Unsupported(message)=>InputError::Unsupported(message.into()),
        _=>InputError::Invariant(format!("direct-card shared calculation failed: {error:?}")),
    }
}

fn selected_enemy<'a>(state:&'a State,target:u64)->Option<&'a super::super::super::model::EnemyState> {
    target.checked_sub(1).and_then(|index|usize::try_from(index).ok())
        .and_then(|index|state.enemies.get(index))
        .filter(|enemy|enemy.life==ActorLife::Active&&enemy.hp>0)
}

fn target_damage(state:&State,target:u64,base:i32)->Result<i32,RunError> {
    super::super::modified_damage(state,target,base)
}

fn damage_all(state:&State,base:i32,work:&mut Vec<(Option<PayloadTarget>,EffectKind)>)->Result<(),RunError> {
    for (index,enemy) in state.enemies.iter().enumerate() {
        if enemy.life!=ActorLife::Active||enemy.hp<=0 { continue; }
        let damage=target_damage(state,index as u64+1,base)?;
        work.push((Some(PayloadTarget::Enemy(enemy.instance_id.clone())),EffectKind::Damage(DamagePayload::Fixed(damage))));
    }
    Ok(())
}

fn channel(work:&mut Vec<(Option<PayloadTarget>,EffectKind)>,orb:OrbKind) {
    work.push((Some(PayloadTarget::Player),EffectKind::Character(C::Channel(orb))));
}

pub(super) fn payload(state:&State,card:&LiveCard,target:Option<u64>)
    ->Result<Option<(i32,Vec<(Option<PayloadTarget>,EffectKind)>)>,RunError>{
    if !is_direct(card.id) { return Ok(None); }
    // Four M16 payloads remain at their original owner; this module only fills
    // their newly required public values.
    if is_legacy(card.id) { return Ok(None); }
    let definition=Catalog::standard().card(card.id).ok_or(RunError::Unsupported("M20 blue catalog row missing"))?;
    let metadata=definition.blue_metadata.ok_or(RunError::Unsupported("M20 blue face metadata missing"))?;
    let face=if card.upgrades>0 { metadata.upgraded } else { metadata.base };
    if (face.target==CardTarget::Enemy)!=target.is_some() { return Err(RunError::InvalidAction); }
    if let Some(target)=target { if selected_enemy(state,target).is_none() { return Err(RunError::InvalidAction); } }

    use CardId::*;
    let up=card.upgrades>0;
    let mut work=Vec::new();
    match card.id {
        ChargeBattery=>{
            work.push((Some(PayloadTarget::Player),EffectKind::GainBlock(super::super::modified_block(state,if up{10}else{7})?)));
            work.push((Some(PayloadTarget::Player),EffectKind::ApplyFullPower{id:FullPowerId("energized_blue"),amount:1}));
        }
        Stack=>{
            let base=card.local.base_block.ok_or(RunError::Unsupported("Stack Block was not refreshed on hand entry"))?;
            work.push((Some(PayloadTarget::Player),EffectKind::GainBlock(super::super::modified_block(state,base)?)));
        }
        Turbo=>{
            work.push((Some(PayloadTarget::Player),EffectKind::GainEnergy(if up{3}else{2})));
            work.push((Some(PayloadTarget::Player),EffectKind::CreateCard{id:CardId::Void,destination:super::super::super::command::CardDestination::Discard,upgrades:0}));
        }
        Aggregate=>work.push((Some(PayloadTarget::Player),EffectKind::Defect(DefectEffect::Direct(DirectEffect::Aggregate{divisor:if up{3}else{4}})))),
        AutoShields=>{
            if state.player.block==0 {
                work.push((Some(PayloadTarget::Player),EffectKind::GainBlock(super::super::modified_block(state,if up{15}else{11})?)));
            }
        }
        Blizzard=>{
            let base=card.local.base_damage.ok_or(RunError::Unsupported("Blizzard damage was not refreshed from Frost history"))?;
            damage_all(state,base,&mut work)?;
        }
        BootSequence=>work.push((Some(PayloadTarget::Player),EffectKind::GainBlock(super::super::modified_block(state,if up{13}else{10})?))),
        Chaos=>work.push((Some(PayloadTarget::Player),EffectKind::Defect(DefectEffect::Direct(DirectEffect::Chaos{requested:if up{2}else{1},orbs:Vec::new()})))),
        Chill=>{
            // In this model `isDeadOrEscaped == false` is a live active actor;
            // HalfDead is excluded by the source helper even at zero HP.
            let count=state.enemies.iter().filter(|enemy|enemy.life==ActorLife::Active&&enemy.hp>0).count() as i32;
            for _ in 0..count { channel(&mut work,OrbKind::Frost); }
        }
        DoomAndGloom=>{
            damage_all(state,if up{14}else{10},&mut work)?;
            channel(&mut work,OrbKind::Dark);
        }
        DoubleEnergy=>work.push((Some(PayloadTarget::Player),EffectKind::Defect(DefectEffect::Direct(DirectEffect::DoubleEnergy)))),
        Glacier=>{
            work.push((Some(PayloadTarget::Player),EffectKind::GainBlock(super::super::modified_block(state,if up{10}else{7})?)));
            channel(&mut work,OrbKind::Frost);
            channel(&mut work,OrbKind::Frost);
        }
        Melter=>work.push((Some(PayloadTarget::SelectedEnemy),EffectKind::Defect(DefectEffect::Direct(DirectEffect::Melter{
            damage:target_damage(state,target.unwrap(),if up{14}else{10})?,
        })))),
        Overclock=>{
            work.push((None,EffectKind::Draw(if up{3}else{2})));
            work.push((Some(PayloadTarget::Player),EffectKind::CreateCard{id:CardId::Burn,destination:super::super::super::command::CardDestination::Discard,upgrades:0}));
        }
        Reprogram=>{
            let amount=if up{2}else{1};
            work.push((Some(PayloadTarget::Player),EffectKind::Character(C::Focus(-amount))));
            work.push((Some(PayloadTarget::Player),EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount}));
            work.push((Some(PayloadTarget::Player),EffectKind::ApplyFullPower{id:FullPowerId("dexterity"),amount}));
        }
        RipAndTear=>work.push((None,EffectKind::Defect(DefectEffect::Direct(DirectEffect::RandomHit{base:if up{9}else{7},remaining:2})))),
        Skim=>work.push((None,EffectKind::Draw(if up{4}else{3}))),
        Sunder=>work.push((Some(PayloadTarget::SelectedEnemy),EffectKind::Defect(DefectEffect::Direct(DirectEffect::Sunder{
            damage:target_damage(state,target.unwrap(),if up{32}else{24})?,
        })))),
        CoreSurge=>{
            work.push((Some(PayloadTarget::SelectedEnemy),EffectKind::Damage(DamagePayload::Fixed(target_damage(state,target.unwrap(),if up{15}else{11})?))));
            work.push((Some(PayloadTarget::Player),EffectKind::ApplyFullPower{id:FullPowerId("artifact"),amount:1}));
        }
        Hyperbeam=>{
            damage_all(state,if up{34}else{26},&mut work)?;
            work.push((Some(PayloadTarget::Player),EffectKind::Character(C::Focus(-3))));
        }
        MeteorStrike=>{
            work.push((Some(PayloadTarget::SelectedEnemy),EffectKind::Damage(DamagePayload::Fixed(target_damage(state,target.unwrap(),if up{30}else{24})?))));
            for _ in 0..3 { channel(&mut work,OrbKind::Plasma); }
        }
        Rainbow=>{
            channel(&mut work,OrbKind::Lightning);
            channel(&mut work,OrbKind::Frost);
            channel(&mut work,OrbKind::Dark);
        }
        ThunderStrike=>{
            let remaining=state.defect.lightning_channeled;
            if remaining>0 {
                work.push((None,EffectKind::Defect(DefectEffect::Direct(DirectEffect::RandomHit{base:if up{9}else{7},remaining}))));
            }
        }
        BeamCell|GoForTheEyes|Leap|SweepingBeam=>unreachable!("legacy row returned before M20 dispatch"),
        _=>return Ok(None),
    }
    Ok(Some((printed_cost(face.cost),work)))
}

fn values(card:&LiveCard,damage:Option<i32>,block:Option<i32>,magic:Option<i32>)->CardPublicValues {
    CardPublicValues { base_damage:damage,base_block:block,magic_number:magic,damage_bonus:None,
        cost_kind:CardCostKind::Fixed,free_play:card.local.free_play||card.local.free_to_play_once,
        exhaust_on_finish:card.local.exhaust_on_finish,ethereal:card.local.ethereal,innate:card.local.innate,
        temporary_cost:card.local.temporary_cost }
}

pub(super) fn public_values(card:&LiveCard)->Option<CardPublicValues>{
    if !is_direct(card.id) { return None; }
    use CardId::*;
    let up=card.upgrades>0;
    let result=match card.id {
        BeamCell=>(Some(if up{4}else{3}),None,Some(if up{2}else{1})),
        ChargeBattery=>(None,Some(if up{10}else{7}),None),
        GoForTheEyes=>(Some(if up{4}else{3}),None,Some(if up{2}else{1})),
        Leap=>(None,Some(if up{12}else{9}),None),
        Stack=>(None,card.local.base_block,None),
        SweepingBeam=>(Some(if up{9}else{6}),None,Some(1)),
        Turbo=>(None,None,Some(if up{3}else{2})),
        Aggregate=>(None,None,None),
        AutoShields=>(None,Some(if up{15}else{11}),None),
        Blizzard=>(card.local.base_damage,None,None),
        BootSequence=>(None,Some(if up{13}else{10}),None),
        Chaos=>(None,None,Some(if up{2}else{1})),
        Chill|DoubleEnergy|Reprogram|Rainbow=>(None,None,None),
        RipAndTear=>(Some(if up{9}else{7}),None,None),
        DoomAndGloom=>(Some(if up{14}else{10}),None,None),
        Glacier=>(None,Some(if up{10}else{7}),Some(2)),
        Melter=>(Some(if up{14}else{10}),None,None),
        Overclock=>(None,None,Some(if up{3}else{2})),
        Skim=>(None,None,Some(if up{4}else{3})),
        Sunder=>(Some(if up{32}else{24}),None,None),
        CoreSurge=>(Some(if up{15}else{11}),None,None),
        Hyperbeam=>(Some(if up{34}else{26}),None,None),
        MeteorStrike=>(Some(if up{30}else{24}),None,Some(3)),
        ThunderStrike=>(Some(if up{9}else{7}),None,card.local.magic_number),
        _=>return None,
    };
    Some(values(card,result.0,result.1,result.2))
}

pub(super) fn refresh_values(card:&mut LiveCard,frost:u32,lightning:u32,discard:usize){
    match card.id {
        CardId::Stack=>card.local.base_block=Some((discard as i32).wrapping_add(if card.upgrades>0{3}else{0})),
        CardId::Blizzard=>card.local.base_damage=Some((frost as i32).wrapping_mul(if card.upgrades>0{3}else{2})),
        CardId::ThunderStrike=>card.local.magic_number=Some(lightning as i32),
        _=>(),
    }
}

pub(super) fn capture(state:&mut State,effects:&mut [Effect])->Result<(),EffectError>{
    for effect in effects {
        let EffectKind::Defect(DefectEffect::Direct(DirectEffect::Chaos{requested,orbs}))=&mut effect.kind else { continue; };
        if !(1..=2).contains(requested)||orbs.len()>usize::from(*requested)
            ||orbs.iter().any(|orb|!matches!(orb,OrbKind::Dark|OrbKind::Frost|OrbKind::Lightning|OrbKind::Plasma)) {
            return Err(InputError::Invariant("Chaos use capture progress bounds".into()).into());
        }
        while orbs.len()<usize::from(*requested) {
            let stream=state.card_rng.as_ref().ok_or_else(||InputError::Invariant("Chaos card-random stream absent".into()))?;
            let (selected,next)=super::super::super::rng::uniform_below(stream,4)?;
            state.card_rng=Some(next);
            orbs.push(match selected {0=>OrbKind::Dark,1=>OrbKind::Frost,2=>OrbKind::Lightning,_=>OrbKind::Plasma});
        }
    }
    Ok(())
}

fn live_source<'a>(state:&'a State,effect:&Effect)->Result<&'a LiveCard,InputError>{
    let Source::Card(owner)=&effect.source else { return Err(InputError::Invariant("M20 direct work requires its card source".into())); };
    state.in_play.as_ref().filter(|card|card.handle()==*owner).and_then(OwnedCard::full)
        .filter(|card|is_direct(card.id))
        .ok_or_else(||InputError::Invariant("M20 direct work requires the matching live card".into()))
}

fn enemy_target<'a>(state:&'a State,effect:&Effect,active:bool)->Result<&'a super::super::super::model::EnemyState,InputError>{
    let Some(Actor::Enemy(owner))=effect.target.as_ref() else { return Err(InputError::Invariant("M20 direct work requires its enemy target".into())); };
    state.enemies.iter().find(|enemy|&enemy.instance_id==owner)
        .filter(|enemy|!active||enemy.life==ActorLife::Active&&enemy.hp>0)
        .ok_or_else(||InputError::Invariant("M20 direct enemy target is not live".into()))
}

pub(super) fn validate_work(state:&State,effect:&Effect,work:&DirectEffect)->Result<(),InputError>{
    if !state.full_runtime { return Err(InputError::Invariant("M20 direct work requires full combat".into())); }
    let card=live_source(state,effect)?;
    use CardId::*;
    let expected=match work {
        DirectEffect::Aggregate{divisor}=>card.id==Aggregate&&*divisor==if card.upgrades>0{3}else{4}
            &&effect.target==Some(Actor::Player),
        DirectEffect::DoubleEnergy=>card.id==DoubleEnergy&&effect.target==Some(Actor::Player),
        DirectEffect::Chaos{requested,orbs}=>card.id==Chaos&&*requested==if card.upgrades>0{2}else{1}
            &&orbs.len()<=usize::from(*requested)&&effect.target==Some(Actor::Player)
            &&orbs.iter().all(|orb|matches!(orb,OrbKind::Dark|OrbKind::Frost|OrbKind::Lightning|OrbKind::Plasma)),
        DirectEffect::RandomHit{base,remaining}=>{
            let valid_card=match card.id {RipAndTear=>*base==if card.upgrades>0{9}else{7}&&*remaining<=2,
                ThunderStrike=>*base==if card.upgrades>0{9}else{7}&&*remaining<=state.defect.lightning_channeled as u32,_=>false};
            valid_card&&*remaining>0&&effect.target.is_none()
        }
        DirectEffect::Melter{damage}=>card.id==Melter&&(0..=30_012).contains(damage)
            &&enemy_target(state,effect,true).is_ok(),
        DirectEffect::Sunder{damage}=>card.id==Sunder&&(0..=30_012).contains(damage)
            &&enemy_target(state,effect,true).is_ok(),
        DirectEffect::SunderCheck=>card.id==Sunder&&enemy_target(state,effect,false).is_ok(),
    };
    if expected { Ok(()) } else { Err(InputError::Invariant("M20 direct continuation producer/target mismatch".into())) }
}

fn queue(state:&mut State,effect:&Effect,target:Option<Actor>,kind:EffectKind){
    state.pending.push_front(Effect{source:effect.source.clone(),target,kind});
}

pub(super) fn apply(state:&mut State,effect:&Effect,work:&DirectEffect)->Result<(),EffectError>{
    validate_work(state,effect,work)?;
    match work {
        DirectEffect::Aggregate{divisor}=>{
            let amount=(state.piles.draw.len() as i32).wrapping_div(i32::from(*divisor));
            queue(state,effect,Some(Actor::Player),EffectKind::GainEnergy(amount));
        }
        DirectEffect::DoubleEnergy=>queue(state,effect,Some(Actor::Player),EffectKind::GainEnergy(state.player.energy)),
        DirectEffect::Chaos{orbs,..}=>for orb in orbs.iter().rev(){queue(state,effect,Some(Actor::Player),EffectKind::Character(C::Channel(*orb)));},
        DirectEffect::RandomHit{base,remaining}=>{
            let Some(owner)=super::super::ironclad::autoplay::capture_blue_random_enemy_target(state)? else { return Ok(()); };
            let (slot,target)=state.enemies.iter().enumerate().find(|(_,enemy)|enemy.instance_id==owner)
                .map(|(slot,enemy)|(slot,enemy.instance_id.clone()))
                .ok_or_else(||InputError::Invariant("captured M20 random target disappeared".into()))?;
            let damage=super::super::modified_damage(state,slot as u64+1,*base).map_err(run_error)?;
            if *remaining>1 {
                queue(state,effect,None,EffectKind::Defect(DefectEffect::Direct(DirectEffect::RandomHit{base:*base,remaining:remaining-1})));
            }
            queue(state,effect,Some(Actor::Enemy(target)),EffectKind::Damage(DamagePayload::Fixed(damage)));
        }
        DirectEffect::Melter{damage}=>{
            let Actor::Enemy(owner)=effect.target.as_ref().unwrap() else { unreachable!("validated Melter target") };
            let enemy=state.enemies.iter_mut().find(|enemy|&enemy.instance_id==owner).ok_or_else(||InputError::Invariant("Melter target disappeared".into()))?;
            enemy.block=0;
            queue(state,effect,Some(Actor::Enemy(owner.clone())),EffectKind::Damage(DamagePayload::Fixed(*damage)));
        }
        DirectEffect::Sunder{damage}=>{
            let target=effect.target.clone();
            queue(state,effect,target.clone(),EffectKind::Defect(DefectEffect::Direct(DirectEffect::SunderCheck)));
            queue(state,effect,target,EffectKind::Damage(DamagePayload::Fixed(*damage)));
        }
        DirectEffect::SunderCheck=>{
            let Actor::Enemy(owner)=effect.target.as_ref().unwrap() else { unreachable!("validated Sunder target") };
            if state.enemies.iter().find(|enemy|&enemy.instance_id==owner).is_some_and(|enemy|enemy.hp<=0) {
                queue(state,effect,Some(Actor::Player),EffectKind::GainEnergy(3));
            }
        }
    }
    Ok(())
}
