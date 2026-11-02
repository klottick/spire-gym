//! Complete source supports and captured requests; delivery uses the shared piles.
use super::*;
use super::model::State;
use crate::content::{CardId,Character};

pub use crate::content::pools::GenerationPool;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum GenerationCost { Ordinary,TurnZero,CombatZero }
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct GenerationWork {
    pub pool:GenerationPool,pub count:u32,pub unique:bool,
    pub destination:CardDestination,pub cost:GenerationCost,pub upgraded:bool,
    pub offer:Option<(bool,u8)>,pub captured:bool,pub cards:Vec<CardId>,pub next:usize,
}

pub(super) fn needs_character(id:CardId)->bool{
    starting_cost(id,0).is_some()
}

/// Thin reader of the M26 pool owner (`content::pools`, design §7.1). Every
/// member must also have a combat constructor before any RNG is used.
pub(super) fn pool(character:Option<Character>,kind:GenerationPool)->Result<Vec<CardId>,InputError>{
    let character=match (character,kind){
        (Some(character),_)=>character,
        // L34 is the same for every character.
        (None,GenerationPool::Colorless)=>Character::Ironclad,
        (None,_)=>return Err(InputError::Unsupported("actual character generator support unavailable".into())),
    };
    let ids=crate::content::pools::cards(crate::content::pools::CardPool::Combat{character,pool:kind})
        .map_err(|_|InputError::Unsupported("complete actual character generator support unavailable".into()))?;
    if ids.iter().any(|id|super::card_state::generated(*id,CardInstanceId(0),0).is_none()){
        return Err(InputError::Unsupported("complete generator constructors unavailable".into()))
    }
    Ok(ids)
}

fn request(pool:GenerationPool,count:u32,destination:CardDestination,cost:GenerationCost,upgraded:bool,offer:Option<(bool,u8)>)->GenerationWork{
    GenerationWork{pool,count,unique:offer.is_some(),destination,cost,upgraded,offer,captured:false,cards:Vec::new(),next:0}
}

pub(super) fn starting_cost(id:CardId,upgrades:u32)->Option<i32>{
    use CardId::*;Some(match id{WhiteNoise|Distraction|Alchemize=>if upgrades>0{0}else{1},Discovery=>1,
        Chrysalis|Metamorphosis=>2,JackOfAllTrades=>0,Transmutation=>0,Magnetism=>if upgrades>0{1}else{2},_=>return None})
}

pub(super) fn payload(state:&State,card:&LiveCard,target:Option<u64>)->Result<Option<(i32,Vec<(Option<cards::PayloadTarget>,EffectKind)>)>,crate::run::RunError>{
    use CardId::*;
    let Some(price)=starting_cost(card.id,card.upgrades) else{return Ok(None)};
    if target.is_some(){return Err(crate::run::RunError::InvalidAction)}
    let upgraded=card.upgrades>0;
    if card.id==Alchemize{
        // M26 §6.2 L: Alchemize gains from the player's class pool for any character under the
        // complete capability; the accepted M15 Silent path is unchanged.
        let complete=super::cards::watcher::complete(state)&&state.silent.source_character.is_some();
        if !(state.silent.source_character==Some(Character::Silent)||complete)||!state.item_context.as_ref().is_some_and(|items|(items.m15_profile()||complete)&&items.valid_potion_capacity()){
            return Err(crate::run::RunError::Unsupported("Alchemize requires actual Silent potion inventory"))
        }
        return Ok(Some((price,vec![(Some(cards::PayloadTarget::Player),EffectKind::Alchemize{proposal:None})])))
    }
    if card.id==Magnetism{return Ok(Some((price,vec![(Some(cards::PayloadTarget::Player),EffectKind::ApplyFullPower{id:FullPowerId("magnetism"),amount:1})])))}
    let work=match card.id{
        WhiteNoise=>request(GenerationPool::Power,1,CardDestination::Hand,GenerationCost::TurnZero,false,None),
        Distraction=>request(GenerationPool::Skill,1,CardDestination::Hand,GenerationCost::TurnZero,false,None),
        Discovery=>request(GenerationPool::Any,3,CardDestination::Hand,GenerationCost::TurnZero,false,Some((false,1))),
        JackOfAllTrades=>request(GenerationPool::Colorless,if upgraded{2}else{1},CardDestination::Hand,GenerationCost::Ordinary,false,None),
        Chrysalis|Metamorphosis=>request(if card.id==Chrysalis{GenerationPool::Skill}else{GenerationPool::Attack},if upgraded{5}else{3},CardDestination::ShuffledDraw,GenerationCost::CombatZero,false,None),
        Transmutation=>request(GenerationPool::Colorless,super::cards::captured_x(state,card)?
            .checked_add(super::cards::chemical_x_bonus(state)?).ok_or(crate::run::RunError::InvalidAction)? as u32,
            CardDestination::Hand,GenerationCost::TurnZero,upgraded,None),
        _=>unreachable!(),
    };
    pool(state.silent.source_character,work.pool).map_err(|_|crate::run::RunError::Unsupported("complete actual-character generation support unavailable"))?;
    Ok(Some((price,vec![(Some(cards::PayloadTarget::Player),EffectKind::Generate(work))])))
}

pub(super) fn public_values(card:&LiveCard)->Option<CardPublicValues>{
    let _=starting_cost(card.id,card.upgrades)?;
    Some(CardPublicValues{base_damage:None,base_block:None,magic_number:match card.id{
        CardId::Chrysalis|CardId::Metamorphosis=>Some(if card.upgrades>0{5}else{3}),CardId::JackOfAllTrades=>Some(if card.upgrades>0{2}else{1}),_=>None},
        damage_bonus:None,cost_kind:if card.id==CardId::Transmutation{CardCostKind::X}else{CardCostKind::Fixed},
        free_play:card.local.free_play,exhaust_on_finish:card.local.exhaust_on_finish,ethereal:card.local.ethereal,
        innate:card.local.innate,temporary_cost:card.local.temporary_cost})
}

fn capture_one(state:&mut State,work:&mut GenerationWork)->Result<(),EffectError>{
    if work.captured{return Ok(())}
    let support=pool(state.silent.source_character,work.pool)?;
    if work.unique&&work.count as usize>support.len(){return Err(InputError::Invariant("generated unique request exceeds support".into()).into())}
    while work.cards.len()<work.count as usize{
        let stream=state.card_rng.as_ref().ok_or_else(||InputError::Invariant("generation random owner absent".into()))?;
        let (index,next)=super::rng::uniform_below(stream,support.len() as u128)?;
        state.card_rng=Some(next);let id=support[index as usize];
        if !work.unique||!work.cards.contains(&id){work.cards.push(id);}
    }
    work.captured=true;Ok(())
}

pub(super) fn capture(state:&mut State,effects:&mut [Effect])->Result<(),EffectError>{
    for effect in effects{
        // Discovery/Transmutation construct their proposals in the queued action.
        // These four cards sample synchronously during their own use method.
        let synchronous=matches!(&effect.source,Source::Card(owner) if state.in_play.as_ref().filter(|card|card.handle()==*owner).or_else(||state.queued_autoplay.get(owner)).and_then(|card|card.full()).is_some_and(|card|matches!(card.id,CardId::WhiteNoise|CardId::Alchemize|CardId::Distraction|CardId::JackOfAllTrades|CardId::Chrysalis|CardId::Metamorphosis)));
        if synchronous{
            if let EffectKind::Generate(work)=&mut effect.kind{capture_one(state,work)?;}
            if let EffectKind::Alchemize{proposal}=&mut effect.kind{
                if proposal.is_none(){
                    let character=state.silent.source_character.ok_or_else(||InputError::Invariant("Alchemize actual character absent".into()))?;
                    let stream=state.card_rng.as_ref().ok_or_else(||InputError::Invariant("Alchemize random owner absent".into()))?;
                    let (id,next)=crate::items::potion_generation::sample(character,true,stream)?;
                    state.card_rng=Some(next);*proposal=Some(id);
                }
            }
        }
    }
    Ok(())
}

pub(super) fn validate(state:&State,effect:&Effect,work:&GenerationWork)->Result<(),InputError>{
    if !state.full_runtime||effect.target!=Some(Actor::Player)||work.count>999
        ||work.next>work.cards.len()||work.cards.len()>work.count as usize
        ||(work.captured&&work.cards.len()!=work.count as usize)
        ||work.offer.is_some_and(|(_,copies)|work.count!=3||!work.unique||!(1..=2).contains(&copies)){
        return Err(InputError::Invariant("generated request shape/cursor".into()))
    }
    super::card_offers::validate_owner(state,&effect.source)?;
    let expected=match &effect.source{
        Source::Card(owner)=>{
            let card=state.in_play.as_ref().filter(|owned|owned.handle()==*owner).and_then(|owned|owned.full())
                .ok_or_else(||InputError::Invariant("generator card owner absent".into()))?;
            payload(state,card,None).ok().flatten().and_then(|(_,effects)|effects.into_iter().find_map(|(_,kind)|match kind{EffectKind::Generate(work)=>Some(work),_=>None}))
        }
        Source::Potion{id,..}=>match id.as_str(){
            "attack_potion"=>Some(request(GenerationPool::Attack,3,CardDestination::Hand,GenerationCost::TurnZero,false,Some((true,potion_copies(state))))),
            "skill_potion"=>Some(request(GenerationPool::Skill,3,CardDestination::Hand,GenerationCost::TurnZero,false,Some((true,potion_copies(state))))),
            "power_potion"=>Some(request(GenerationPool::Power,3,CardDestination::Hand,GenerationCost::TurnZero,false,Some((true,potion_copies(state))))),
            "colorless_potion"=>Some(request(GenerationPool::Colorless,3,CardDestination::Hand,GenerationCost::TurnZero,false,Some((false,potion_copies(state))))),_=>None},
        Source::Actor(Actor::Player)=>{
            let id=match work.pool{GenerationPool::Colorless=>"magnetism",GenerationPool::Common=>"hello_world",GenerationPool::Power=>"creative_ai",_=>""};
            state.player_full_powers.iter().any(|power|power.id.0==id&&power.state.amount>0)
                .then(||request(work.pool,1,CardDestination::Hand,GenerationCost::Ordinary,false,None))
        },
        Source::Relic(crate::content::RelicId::Enchiridion)=>
            Some(request(GenerationPool::Power,1,CardDestination::Hand,GenerationCost::TurnZero,false,None)),
        Source::Relic(crate::content::RelicId::DeadBranch)=>
            Some(request(GenerationPool::Any,1,CardDestination::Hand,GenerationCost::Ordinary,false,None)),
        // M27 seam: Toolbox and Nilry's Codex own distinct, source-qualified offers.
        Source::Relic(crate::content::RelicId::Toolbox)=>
            Some(request(GenerationPool::Colorless,3,CardDestination::Hand,GenerationCost::Ordinary,false,Some((false,1)))),
        Source::Relic(crate::content::RelicId::NilrysCodex)=>
            Some(request(GenerationPool::Any,3,CardDestination::ShuffledDraw,GenerationCost::Ordinary,false,Some((true,1)))),
        _=>None,
    }.ok_or_else(||InputError::Invariant("generator source/request owner mismatch".into()))?;
    let mut shape=work.clone();shape.captured=false;shape.cards.clear();shape.next=0;
    if shape!=expected{return Err(InputError::Invariant("generator payload differs from source request".into()))}

    let support=pool(state.silent.source_character,work.pool)?;
    if work.cards.iter().any(|id|!support.contains(id))||work.unique&&work.cards.iter().enumerate().any(|(i,id)|work.cards[..i].contains(id)){
        return Err(InputError::Invariant("generated request outside complete support".into()))
    }
    Ok(())
}

pub(super) fn apply(state:&mut State,effect:&Effect,request:&GenerationWork)->Result<(),EffectError>{
    let mut work=request.clone();
    let capture_result=capture_one(state,&mut work);
    if let Some(current)=state.current.as_mut(){current.kind=EffectKind::Generate(work.clone());}
    capture_result?;
    if let Some((optional,copies))=work.offer{
        state.pending.push_front(Effect{source:effect.source.clone(),target:Some(Actor::Player),kind:EffectKind::CardOffer{cards:work.cards,optional,copies,zero_cost:work.cost!=GenerationCost::Ordinary}});
        return Ok(())
    }
    for index in work.next..work.cards.len(){
        let instance=super::piles::next_generated_instance(state)?;
        let white_noise=matches!(&effect.source,Source::Card(owner) if state.in_play.as_ref().filter(|card|card.handle()==*owner).and_then(|card|card.full()).is_some_and(|card|card.id==CardId::WhiteNoise));
        let mut template=creation_template(state,&work,work.cards[index],white_noise)?;
        let mut card=super::cards::make_stat_equivalent_copy(&mut template,instance,super::cards::copy_history(state))?;
        if !white_noise&&!card.local.x_cost&&!matches!(card.id,CardId::Reflex|CardId::Tactician){
            match work.cost{GenerationCost::Ordinary=>(),GenerationCost::TurnZero=>card.local.turn_cost=Some(0),
                GenerationCost::CombatZero=>{card.local.mutable_combat_cost=Some(0);card.local.turn_cost=Some(0);card.local.temporary_cost=0;}}
        }
        super::piles::insert_generated(state,card,work.destination)?;
        work.next=index+1;
        if let Some(current)=state.current.as_mut(){current.kind=EffectKind::Generate(work.clone());}
    }
    Ok(())
}

/// One creation checkpoint, also accepting the existing typed upgraded request.
/// WhiteNoise assigns turn cost before the creation action upgrades its template.
pub(super) fn creation_template(state:&State,work:&GenerationWork,id:CardId,white_noise:bool)->Result<LiveCard,EffectError>{
    let mut template=super::card_state::generated_with_history(id,CardInstanceId(0),0,super::card_state::CardConstructionHistory::from_state(state))
        .ok_or_else(||InputError::Invariant("generated constructor absent".into()))?;
    if white_noise&&!template.local.x_cost{template.local.turn_cost=Some(0);}
    if work.upgraded{super::cards::ironclad::rules::upgrade(&mut template).map_err(|_|InputError::Invariant("generated upgrade unavailable".into()))?;}
    Ok(template)
}

pub(super) fn start_turn(state:&State,hook:Hook,target:Option<&Actor>,work:&mut Vec<Effect>){
    // M20's required L34 callback uses source-time capture below. The frozen
    // older profiles retain their existing deferred-generation checkpoint.
    if state.item_context.as_ref().is_some_and(|items|items.m20_profile()){return}
    if hook!=Hook::TurnStartedPreDraw||target!=Some(&Actor::Player)||!state.enemies.iter().any(|enemy|enemy.hp>0&&enemy.life==ActorLife::Active){return}
    let count=state.player_full_powers.iter().find(|power|power.id.0=="magnetism").map_or(0,|power|power.state.amount);
    for _ in 0..count{work.push(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Generate(request(GenerationPool::Colorless,1,CardDestination::Hand,GenerationCost::Ordinary,false,None))});}
}

pub(super) fn upgrade_local(card:&mut LiveCard)->Result<(),crate::run::RunError>{
    match card.id{CardId::Distraction|CardId::Alchemize=>super::card_state::upgrade_base_cost(card,1,0)?,
        CardId::Magnetism=>super::card_state::upgrade_base_cost(card,2,1)?,
        CardId::Discovery=>card.local.exhaust_on_finish=false,_=>()}
    Ok(())
}

pub(crate) fn offer_request(pool:GenerationPool,optional:bool,copies:u8)->EffectKind{
    EffectKind::Generate(request(pool,3,CardDestination::Hand,GenerationCost::TurnZero,false,Some((optional,copies))))
}

fn potion_copies(state:&State)->u8{if state.item_context.as_ref().is_some_and(|items|items.owns_relic(crate::content::RelicId::SacredBark)){2}else{1}}

fn not_basically_dead(state:&State)->bool{
    state.enemies.iter().any(|enemy|enemy.life==ActorLife::HalfDead||enemy.life==ActorLife::Active&&enemy.hp>0)
}

/// M20 required start callbacks select now; generated delivery remains queued.
/// Older-profile Magnetism invocation/capture behavior is unchanged.
pub(super) fn blue_start_turn(state:&mut State,hook:Hook,target:Option<&Actor>,work:&mut Vec<Effect>)->Result<(),EffectError>{
    // M26 §6.3 L / F13: Hello World and Creative AI run for any holder under the complete capability
    // (player's pool). Magnetism's M20 branch stays C: other contexts keep the legacy start_turn path.
    if hook!=Hook::TurnStartedPreDraw||target!=Some(&Actor::Player)
        ||state.silent.source_character!=Some(Character::Defect)&&!super::cards::watcher::complete(state){return Ok(())}
    for power in state.player_full_powers.clone(){
        let pool=match power.id.0 {
            "hello_world" if not_basically_dead(state)=>GenerationPool::Common,
            "creative_ai"=>GenerationPool::Power,
            "magnetism" if state.item_context.as_ref().is_some_and(|items|items.m20_profile())&&not_basically_dead(state)=>GenerationPool::Colorless,
            _=>continue,
        };
        for _ in 0..power.state.amount.max(0){
            let mut request=request(pool,1,CardDestination::Hand,GenerationCost::Ordinary,false,None);
            let result=capture_one(state,&mut request);
            let effect=Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Generate(request)};
            if let Err(error)=result{
                for accepted in work.drain(..).rev(){state.pending.push_front(accepted);}
                state.current=Some(effect);return Err(error)
            }
            work.push(effect);
        }
    }Ok(())
}
