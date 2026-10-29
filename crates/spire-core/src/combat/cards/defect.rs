//! Finite blue producers over the shared card, orb and action owners.
use super::*;
mod direct;
mod state;
mod powers;
use crate::content::CardId;
use crate::run::RunError;
use crate::combat::characters::{CharacterEffect as C,OrbKind,OrbPassiveGroup};

#[derive(Clone,Debug,PartialEq,Eq)]
pub enum DefectEffect {
    State(state::StateEffect),
    Direct(direct::DirectEffect),
    EssenceOfDarkness { potency:i32 },
    GeneticGrowth { amount:i32 }, ClawGrowth, SteamWear, StreamlineCost,
    Repeat { child:CardHandle,target:Option<Actor>,energy_on_use:i32,cancelled:bool },
}
#[derive(Clone,Default,PartialEq,Eq)]
pub struct DefectState {
    pub echo_doubled_this_turn:u32,
    pub frost_channeled:u32,
    pub lightning_channeled:u32,
    pub rebound_just_applied:bool,
    pub permanent_mutations:Vec<super::super::PermanentCardMutation>,
}

// New derived history is private; retain the accepted Debug transcript exactly.
impl std::fmt::Debug for DefectState {
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result {
        f.debug_struct("DefectState").field("echo_doubled_this_turn",&self.echo_doubled_this_turn)
            .field("permanent_mutations",&self.permanent_mutations).finish()
    }
}

pub(in crate::combat) fn starting_cost(id:CardId,upgrades:u32)->Option<i32>{
    if let Some(metadata)=crate::content::Catalog::standard().card(id).and_then(|row|row.blue_metadata){
        return Some(match if upgrades>0{metadata.upgraded.cost}else{metadata.base.cost}{crate::content::CardCost::Energy(n)=>i32::from(n),crate::content::CardCost::X=>0})
    }
    use CardId::*;let up=upgrades>0;
    Some(match id {
        Zap|Dualcast|Recursion=>if up{0}else{1},Fusion=>if up{1}else{2},
        Consume|Electrodynamics|Streamline=>2,EchoForm=>3,
        Fission|Claw|SteamBarrier=>0,
        StrikeBlue|DefendBlue|BallLightning|ColdSnap|Coolheaded|Barrage|CompileDriver|Darkness|Capacitor|Defragment|BiasedCognition|Loop|Bullseye|StaticDischarge|Storm|Heatsinks|Amplify|GeneticAlgorithm=>1,
        _=>return None,
    })
}

pub(in crate::combat) fn initialize(card:&mut LiveCard){
    use CardId::*;let up=card.upgrades>0;
    if let Some(metadata)=crate::content::Catalog::standard().card(card.id).and_then(|row|row.blue_metadata){
        let face=if up{metadata.upgraded}else{metadata.base};
        card.local.exhaust_on_finish=face.keywords.exhaust;card.local.innate=face.keywords.innate;
        card.local.ethereal=face.keywords.ethereal;card.local.x_cost=face.cost==crate::content::CardCost::X;
    }
    match card.id {
        Claw=>card.local.base_damage=Some(if up{5}else{3}),
        SteamBarrier=>card.local.base_block=Some(if up{8}else{6}),
        GeneticAlgorithm=>card.local.exhaust_on_finish=true,
        Fission=>card.local.exhaust_on_finish=true,
        EchoForm=>card.local.ethereal=!up,
        Storm=>card.local.innate=up,
        _=>(),
    }
}

pub(in crate::combat) fn upgrade_local(card:&mut LiveCard)->Result<(),RunError>{
    if !crate::content::defect::supports_m20_permanent(card.id){return Ok(())}
    match card.id {
        CardId::Claw=>card.local.base_damage=Some(card.local.base_damage.unwrap_or(3).wrapping_add(2)),
        CardId::SteamBarrier=>card.local.base_block=Some(card.local.base_block.unwrap_or(6).wrapping_add(2)),
        CardId::EchoForm=>card.local.ethereal=false,
        CardId::Storm=>card.local.innate=true,
        _=>(),
    }
    if let Some(metadata)=crate::content::Catalog::standard().card(card.id).and_then(|row|row.blue_metadata){
        card.local.exhaust_on_finish=metadata.upgraded.keywords.exhaust;
        card.local.innate=metadata.upgraded.keywords.innate;card.local.ethereal=metadata.upgraded.keywords.ethereal;
    }
    let old=starting_cost(card.id,0).unwrap();let next=starting_cost(card.id,1).unwrap();
    if old!=next {super::super::card_state::upgrade_base_cost(card,old,next)?;}
    Ok(())
}

pub(in crate::combat) fn payload(state:&State,card:&LiveCard,target:Option<u64>)
    ->Result<Option<(i32,Vec<(Option<PayloadTarget>,EffectKind)>)>,RunError>{
    if let Some(result)=powers::payload(state,card,target)?{return Ok(Some(result))}
    if let Some(result)=state::payload(state,card,target)?{return Ok(Some(result))}
    if let Some(result)=direct::payload(state,card,target)?{return Ok(Some(result))}
    use CardId::*;let up=card.upgrades>0;
    if !matches!(card.id,Darkness|Fusion|Capacitor|Consume|Defragment|BiasedCognition|Loop|Electrodynamics|Bullseye|Recursion|Fission|StaticDischarge|Storm|Heatsinks|Amplify|EchoForm|GeneticAlgorithm|Claw|SteamBarrier|Streamline){return Ok(None)}
    if matches!(card.id,Bullseye|Claw|Streamline)!=target.is_some(){return Err(RunError::InvalidAction)}
    let mut work=Vec::new();
    let player=Some(PayloadTarget::Player);
    let char_work=|action|(Some(PayloadTarget::Player),EffectKind::Character(action));
    let power=|id,amount|(Some(PayloadTarget::Player),EffectKind::ApplyFullPower{id:FullPowerId(id),amount});
    match card.id {
        Darkness=>{work.push(char_work(C::Channel(OrbKind::Dark)));if up{work.push(char_work(C::PassiveGroup(OrbPassiveGroup::Dark)));}},
        Fusion=>work.push(char_work(C::Channel(OrbKind::Plasma))),
        Capacitor=>work.push(char_work(C::Slots(if up{3}else{2}))),
        Consume=>{work.push(char_work(C::Focus(if up{3}else{2})));work.push(char_work(C::Slots(-1)));},
        Defragment=>work.push(char_work(C::Focus(if up{2}else{1}))),
        BiasedCognition=>{work.push(char_work(C::Focus(if up{5}else{4})));work.push(power("bias",1));},
        Loop=>work.push(power("loop",if up{2}else{1})),
        Electrodynamics=>{work.push(power("electrodynamics",1));for _ in 0..if up{3}else{2}{work.push(char_work(C::Channel(OrbKind::Lightning)));}},
        Bullseye=>{
            work.push((Some(PayloadTarget::SelectedEnemy),EffectKind::Damage(DamagePayload::Fixed(modified_damage(state,target.unwrap(),if up{11}else{8})?))));
            work.push((Some(PayloadTarget::SelectedEnemy),EffectKind::ApplyFullPower{id:FullPowerId("lock_on"),amount:if up{3}else{2}}));
        }
        Recursion=>work.push(char_work(C::Recursion)),
        Fission=>work.push(char_work(C::Fission{evoke:up})),
        StaticDischarge=>work.push(power("static_discharge",if up{2}else{1})),
        Storm=>work.push(power("storm",1)),
        Heatsinks=>work.push(power("heatsinks",if up{2}else{1})),
        Amplify=>work.push(power("amplify",if up{2}else{1})),
        EchoForm=>work.push(power("echo_form",1)),
        GeneticAlgorithm=>{
            let block=modified_mutable_block(state,card.misc)?;
            work.push((player.clone(),EffectKind::Defect(DefectEffect::GeneticGrowth{amount:if up{3}else{2}})));
            work.push((player,EffectKind::GainBlock(block)));
        }
        Claw|Streamline=>{
            let base=if card.id==Claw{card.local.base_damage.unwrap_or(if up{5}else{3})}else{if up{20}else{15}};
            work.push((Some(PayloadTarget::SelectedEnemy),EffectKind::Damage(DamagePayload::Fixed(modified_mutable_damage(state,target.unwrap(),base)?))));
            work.push((None,EffectKind::Defect(if card.id==Claw{DefectEffect::ClawGrowth}else{DefectEffect::StreamlineCost})));
        }
        SteamBarrier=>{
            work.push((player,EffectKind::GainBlock(modified_mutable_block(state,card.local.base_block.unwrap_or(if up{8}else{6}))?)));
            work.push((None,EffectKind::Defect(DefectEffect::SteamWear)));
        }
        _=>unreachable!(),
    }
    Ok(Some((starting_cost(card.id,card.upgrades).unwrap(),work)))
}

pub(in crate::combat) fn public_values(card:&LiveCard)->Option<CardPublicValues>{
    if let Some(values)=powers::public_values(card){return Some(values)}
    if let Some(values)=state::public_values(card){return Some(values)}
    if let Some(values)=direct::public_values(card){return Some(values)}
    use CardId::*;let up=card.upgrades>0;
    let (damage,block,magic)=match card.id {
        StrikeBlue=>(Some(if up{9}else{6}),None,None),
        DefendBlue=>(None,Some(if up{8}else{5}),None),
        BallLightning=>(Some(if up{10}else{7}),None,None),
        ColdSnap=>(Some(if up{9}else{6}),None,None),
        Barrage=>(Some(if up{6}else{4}),None,None),
        CompileDriver=>(Some(if up{10}else{7}),None,None),
        Coolheaded=>(None,None,Some(if up{2}else{1})),
        Bullseye=>(Some(if up{11}else{8}),None,Some(if up{3}else{2})),
        Claw=>(Some(card.local.base_damage.unwrap_or(if up{5}else{3})),None,Some(2)),
        SteamBarrier=>(None,Some(card.local.base_block.unwrap_or(if up{8}else{6})),None),
        Streamline=>(Some(if up{20}else{15}),None,None),
        GeneticAlgorithm=>(None,Some(card.misc),Some(if up{3}else{2})),
        Capacitor=>(None,None,Some(if up{3}else{2})),
        Consume=>(None,None,Some(if up{3}else{2})),
        Defragment|Loop|StaticDischarge|Heatsinks|Amplify=>(None,None,Some(if up{2}else{1})),
        BiasedCognition=>(None,None,Some(if up{5}else{4})),
        Electrodynamics=>(None,None,Some(if up{3}else{2})),
        Zap|Dualcast|Darkness|Fusion|Recursion|Fission|Storm|EchoForm=>(None,None,None),
        _=>return None,
    };
    Some(CardPublicValues{base_damage:damage,base_block:block,magic_number:magic,damage_bonus:None,cost_kind:CardCostKind::Fixed,
        free_play:card.local.free_play||card.local.free_to_play_once,exhaust_on_finish:card.local.exhaust_on_finish,
        ethereal:card.local.ethereal,innate:card.local.innate,temporary_cost:card.local.temporary_cost})
}

pub(in crate::combat) fn validate_work(state:&State,effect:&Effect,work:&DefectEffect)->Result<(),InputError>{
    if let DefectEffect::State(work)=work{return state::validate_work(state,effect,work)}
    if let DefectEffect::Direct(work)=work{return direct::validate_work(state,effect,work)}
    if let DefectEffect::EssenceOfDarkness{potency}=work {
        return if state.full_runtime&&effect.target==Some(Actor::Player)
            &&matches!(&effect.source,Source::Potion{slot,id:crate::content::PotionId::EssenceOfDarkness} if state.item_context.as_ref().is_some_and(|items|
                items.potions().get(*slot).is_some_and(|potion|potion.is_none()||*potion==Some(crate::content::PotionId::EssenceOfDarkness))&&*potency==if items.owns_relic(crate::content::RelicId::SacredBark){2}else{1}))
            {Ok(())}else{Err(InputError::Invariant("Essence of Darkness source/potency".into()))}
    }
    if let DefectEffect::Repeat{child,target,energy_on_use,..}=work {
        let Source::Card(parent)=&effect.source else{return Err(InputError::Invariant("blue repeat parent source".into()))};
        let retired_parent=state.queued_autoplay.get(child).and_then(|c|c.full()).is_some_and(|copy|
            super::super::piles::owned_full_cards(state).any(|card|super::super::card_state::handle(card.instance)==*parent&&card.id==copy.id&&card.copy_family==copy.copy_family));
        let owned=retired_parent||state.in_play.as_ref().is_some_and(|card|card.handle()==*parent)||state.play_frames.iter().any(|frame|
            matches!(&frame.parent_owner,super::super::model::PlayFrameParentOwner::Card(owner) if owner==parent)&&frame.child==*child&&frame.parent_finish_checkpoint_done);
        if !state.full_runtime||effect.target.is_some()||!owned||!state.queued_autoplay.contains_key(child)||*energy_on_use< -1
            ||target.as_ref().is_some_and(|target|matches!(target,Actor::Enemy(id) if !state.enemies.iter().any(|e|&e.instance_id==id))) {
            return Err(InputError::Invariant("blue repeat retained custody".into()))
        }
        return Ok(())
    }
    let Some(card)=state.in_play.as_ref().filter(|owned|effect.source==Source::Card(owned.handle())).and_then(|c|c.full()) else {
        return Err(InputError::Invariant("blue mutation requires in-play owner".into()))
    };
    let valid=match work {
        DefectEffect::GeneticGrowth{amount}=>card.id==CardId::GeneticAlgorithm&&*amount==if card.upgrades>0{3}else{2},
        DefectEffect::ClawGrowth=>card.id==CardId::Claw,
        DefectEffect::SteamWear=>card.id==CardId::SteamBarrier,
        DefectEffect::StreamlineCost=>card.id==CardId::Streamline,
        _=>false,
    };
    if !valid{return Err(InputError::Invariant("blue mutation producer".into()))}Ok(())
}

pub(in crate::combat) fn apply(state:&mut State,effect:&Effect,work:&DefectEffect)->Result<(),EffectError>{
    validate_work(state,effect,work)?;
    if let DefectEffect::State(work)=work{return state::apply(state,effect,work)}
    if let DefectEffect::Direct(work)=work{return direct::apply(state,effect,work)}
    if let DefectEffect::Repeat{child,target,energy_on_use,cancelled}=work {
        return super::ironclad::autoplay::apply_shared_repeat(state,effect,child,target,*energy_on_use,*cancelled)
    }
    if let DefectEffect::EssenceOfDarkness{potency}=work {
        if !matches!(&effect.source,Source::Potion{slot,..} if state.item_context.as_ref().is_some_and(|items|items.potions().get(*slot).is_some_and(Option::is_none))){return Err(InputError::Invariant("Essence source was not vacated".into()).into())}
        for _ in 0..state.character.orb_slots.len()*(*potency as usize) {
            state.pending.push_front(Effect{source:effect.source.clone(),target:Some(Actor::Player),kind:EffectKind::Character(C::Channel(OrbKind::Dark))});
        }
        return Ok(())
    }
    let source=state.in_play.as_ref().unwrap().full().unwrap().clone();
    match work {
        DefectEffect::GeneticGrowth{amount}=>grow_genetic(state,&source,*amount)?,
        DefectEffect::ClawGrowth=>{
            for owned in state.piles.hand.iter_mut().chain(&mut state.piles.draw).chain(&mut state.piles.discard).chain(state.in_play.iter_mut()) {
                if let Some(card)=owned.full_mut().filter(|card|card.id==CardId::Claw) {
                    card.local.base_damage=Some(card.local.base_damage.unwrap_or(if card.upgrades>0{5}else{3}).wrapping_add(2));
                }
            }
        }
        DefectEffect::SteamWear|DefectEffect::StreamlineCost=>{
            super::super::piles::for_each_owned_card_mut(state,|card|if card.copy_family==source.copy_family&&card.id==source.id {
                if matches!(work,DefectEffect::SteamWear) {
                    card.local.base_block=Some(card.local.base_block.unwrap_or(if card.upgrades>0{8}else{6}).wrapping_sub(1));
                }else {
                    let base=card.local.mutable_combat_cost.or(card.local.randomized_cost).unwrap_or(2);
                    let turn=card.local.turn_cost.unwrap_or(base.saturating_add(card.local.temporary_cost));
                    if turn>0 {let next=turn.saturating_sub(1).max(0);card.local.turn_cost=Some(next);card.local.mutable_combat_cost=Some(next);}
                    else {card.local.mutable_combat_cost=Some(base.saturating_sub(1).max(0));card.local.turn_cost=Some(0);}
                    card.local.temporary_cost=0;
                }
            });
        }
        _=>(),
    }Ok(())
}

fn grow_genetic(state:&mut State,source:&LiveCard,amount:i32)->Result<(),InputError>{
    use std::collections::BTreeSet;
    let mut seen=BTreeSet::new();let mut permanent=None;let mut invalid=false;
    super::super::piles::for_each_owned_card_mut(state,|card|{
        if !seen.insert(card.instance){invalid=true;}
        if card.copy_family==source.copy_family {
            if card.id!=CardId::GeneticAlgorithm {invalid=true;}
            if let CombatCardRef::Permanent(master)=card.origin {
                if master!=card.instance||master.0!=source.copy_family||permanent.is_some(){invalid=true;}
                permanent=Some((master,card.misc.wrapping_add(amount)));
            }
        }
    });
    if invalid||matches!(source.origin,CombatCardRef::Permanent(_))&&permanent.is_none(){return Err(InputError::Invariant("Genetic family physical ownership".into()))}
    let mutation=if let Some((instance,misc))=permanent {
        let ordinal=u64::try_from(state.defect.permanent_mutations.len()).ok().and_then(|n|n.checked_add(1)).ok_or_else(||InputError::Unsupported("permanent mutation ordinal exhausted".into()))?;
        state.defect.permanent_mutations.try_reserve(1).map_err(|_|InputError::Unsupported("permanent mutation allocation exhausted".into()))?;
        Some(super::super::PermanentCardMutation{ordinal,instance,id:CardId::GeneticAlgorithm,misc})
    }else{None};
    super::super::piles::for_each_owned_card_mut(state,|card|if card.copy_family==source.copy_family&&card.id==CardId::GeneticAlgorithm {card.misc=card.misc.wrapping_add(amount);});
    if let Some(mutation)=mutation{state.defect.permanent_mutations.push(mutation);}Ok(())
}

pub(in crate::combat) fn valid_focus_application(state:&State,source:&Source,amount:i32)->bool{
    match source {
        Source::Card(owner)=>state.in_play.as_ref().filter(|c|c.handle()==*owner).and_then(|c|c.full()).is_some_and(|card|amount==match card.id {
            CardId::Reprogram=>if card.upgrades>0{-2}else{-1},
            CardId::Hyperbeam=>-3,
            CardId::Consume=>if card.upgrades>0{3}else{2},
            CardId::Defragment=>if card.upgrades>0{2}else{1},
            CardId::BiasedCognition=>if card.upgrades>0{5}else{4},
            _=>return false,
        }),
        Source::Actor(Actor::Player)=>state.player_full_powers.iter().any(|p|p.id.0=="bias"&&p.state.amount>0&&amount== -p.state.amount),
        Source::Potion{slot,id:crate::content::PotionId::FocusPotion}=>state.item_context.as_ref().is_some_and(|items|
            items.potions().get(*slot).is_some_and(|p|p.is_none()||*p==Some(crate::content::PotionId::FocusPotion))
                &&amount==if items.owns_relic(crate::content::RelicId::SacredBark){4}else{2}),
        Source::None=>amount==1&&state.item_context.as_ref().is_some_and(|items|items.owns_relic(crate::content::RelicId::DataDisk)),
        _=>false,
    }
}

pub(in crate::combat) fn card_played(state:&mut State,source:&Source,target:Option<&Actor>)->Result<(),EffectError>{
    let Source::Card(owner)=source else{return Ok(())};
    let Some(card)=state.in_play.as_ref().filter(|c|c.handle()==*owner).and_then(|c|c.full()).cloned() else{return Ok(())};
    let is_power=crate::content::Catalog::standard().card(card.id).is_some_and(|d|d.card_type==crate::content::CardType::Power);
    // Capture existing listeners before the new Power installs its own effects.
    if is_power {
        for owned in state.piles.hand.iter_mut().chain(&mut state.piles.draw).chain(&mut state.piles.discard){
            if let Some(card)=owned.full_mut().filter(|card|card.id==CardId::ForceField){crate::combat::card_state::discount_force_field(card);}
        }
        let listeners=state.player_full_powers.clone();
        for power in listeners {
            if power.state.amount<=0{continue}
            match power.id.0 {
                "storm"=>for _ in 0..power.state.amount {super::super::enemies::act2::bottom(state,Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Character(C::Channel(OrbKind::Lightning))});},
                "heatsinks"=>state.pending.push_front(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Draw(power.state.amount)}),
                _=>(),
            }
        }
    }
    if card.local.purge||state.play_frames.last().is_some_and(|frame|frame.child==*owner&&frame.disposition==super::super::model::FinishDisposition::Purge){return Ok(())}
    let energy=state.play_context.as_ref().filter(|c|c.card==*owner).map_or(state.player.energy,|c|c.energy_on_use);
    let echo=state.player_full_powers.iter().find(|p|p.id.0=="echo_form").map_or(0,|p|p.state.amount);
    let position=i64::try_from(state.silent.played_this_turn.len()).map_err(|_|InputError::Unsupported("Echo turn position exhausted".into()))?;
    if echo>0&&position-i64::from(state.defect.echo_doubled_this_turn)<=i64::from(echo){
        let next=state.defect.echo_doubled_this_turn.checked_add(1).ok_or_else(||InputError::Unsupported("Echo counter exhausted".into()))?;
        super::ironclad::autoplay::queue_blue_repeat(state,source,target,energy)?;
        state.defect.echo_doubled_this_turn=next;
    }
    if is_power {
        if let Some(index)=state.player_full_powers.iter().position(|p|p.id.0=="amplify"&&p.state.amount>0){
            super::ironclad::autoplay::queue_blue_repeat(state,source,target,energy)?;
            state.player_full_powers[index].state.amount-=1;
            if state.player_full_powers[index].state.amount==0{state.player_full_powers.remove(index);}
        }
    }
    Ok(())
}

pub(in crate::combat) fn turn_started(state:&mut State,work:&mut Vec<Effect>)->Result<(),EffectError>{
    if state.silent.source_character!=Some(crate::content::Character::Defect){return Ok(())}
    use crate::combat::characters::{self,OrbPassivePhase};
    state.defect.echo_doubled_this_turn=0;
    let source=Source::Actor(Actor::Player);
    // Relic callback invocation queues Impulse before the powers' queued work.
    if let Some(items)=state.item_context.as_mut().filter(|items|items.emotion_chip_armed){
        items.emotion_chip_armed=false;
        work.push(Effect{source:source.clone(),target:Some(Actor::Player),kind:EffectKind::Character(C::PassiveGroup(OrbPassiveGroup::All))});
    }
    for power in state.player_full_powers.clone(){
        match power.id.0 {
            "loop"=>for _ in 0..power.state.amount.max(0){
                for phase in [OrbPassivePhase::Start,OrbPassivePhase::End]{if let Some(effect)=characters::passive_work(state,&source,0,phase)?{work.push(effect);}}
            },
            "bias" if power.state.amount>0=>work.push(Effect{source:source.clone(),target:Some(Actor::Player),kind:EffectKind::Character(C::Focus(-power.state.amount))}),
            _=>(),
        }
    }
    for index in 0..state.character.orb_slots.len(){if let Some(effect)=characters::passive_work(state,&source,index,OrbPassivePhase::Start)?{work.push(effect);}}
    if state.item_context.as_ref().is_some_and(|items|items.owns_relic(crate::content::RelicId::GoldPlatedCables)) {
        if let Some(effect)=characters::passive_work(state,&source,0,OrbPassivePhase::Start)?{work.push(effect);}
    }
    // Later-turn DrawCardAction construction recharges synchronously after hook
    // invocation, before any of the queued energy actions above execute.
    if state.turn>1 {
        state.player.energy=state.player.max_energy;
        for id in ["energized","energized_blue"] {
            if let Some(power)=state.player_full_powers.iter().find(|p|p.id.0==id&&p.state.amount>0).cloned(){
                state.player.energy=state.player.energy.saturating_add(power.state.amount).min(999);
                work.push(Effect{source:source.clone(),target:Some(Actor::Player),kind:EffectKind::RemoveFullPower(power.id)});
            }
        }
    }
    Ok(())
}
pub(in crate::combat) fn on_attacked(state:&mut State,effect:&Effect,payload:&DamagePayload,amount:i32){
    if amount<=0||!matches!(effect.source,Source::Actor(Actor::Enemy(_)))
        ||!matches!(payload,DamagePayload::Fixed(_)|DamagePayload::EnemyMove(_)|DamagePayload::FullEnemyMove(_)){return}
    let count=state.player_full_powers.iter().find(|p|p.id.0=="static_discharge").map_or(0,|p|p.state.amount);
    for _ in 0..count.max(0){
        state.pending.push_front(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Character(C::Channel(OrbKind::Lightning))});
    }
}

pub(in crate::combat) fn validate_mutations(state:&State)->Result<(),InputError>{
    let mut latest=std::collections::BTreeMap::new();
    for (index,mutation) in state.defect.permanent_mutations.iter().enumerate(){
        if mutation.ordinal!=index as u64+1||mutation.id!=CardId::GeneticAlgorithm{return Err(InputError::Invariant("permanent mutation ordinal or kind".into()))}
        latest.insert(mutation.instance,mutation.misc);
    }
    let mut seen=std::collections::BTreeSet::new();
    for card in super::super::piles::owned_full_cards(state){
        if let CombatCardRef::Permanent(instance)=card.origin {
            if let Some(misc)=latest.get(&instance){
                if !seen.insert(instance)||card.id!=CardId::GeneticAlgorithm||card.instance!=instance||card.copy_family!=instance.0||card.misc!=*misc {
                    return Err(InputError::Invariant("permanent mutation live owner mismatch".into()))
                }
            }
        }
    }
    if seen.len()!=latest.len(){return Err(InputError::Invariant("permanent mutation master owner missing".into()))}Ok(())
}

pub(in crate::combat) fn valid_orb_source(state:&State,source:&Source)->bool{
    let m20=state.silent.source_character==Some(crate::content::Character::Defect)
        &&state.item_context.as_ref().is_some_and(|items|items.m20_profile());
    state.full_runtime&&match source {
        Source::Actor(Actor::Player)=>state.silent.source_character==Some(crate::content::Character::Defect),
        Source::Card(owner)=>super::super::piles::owned_full_cards(state).any(|card|
            super::super::card_state::handle(card.instance)==*owner&&(crate::content::defect::supports_permanent(card.id)
                ||m20&&crate::content::defect::supports_m20_permanent(card.id))),
        Source::Potion{slot,id:crate::content::PotionId::EssenceOfDarkness}=>m20
            &&state.item_context.as_ref().is_some_and(|items|items.potions().get(*slot).is_some_and(Option::is_none)),
        _=>false,
    }
}

pub(in crate::combat) fn validate_character(state:&State,effect:&Effect,action:C)->Result<(),InputError>{
    let actual=match &effect.source {Source::Card(owner)=>state.in_play.as_ref().filter(|c|c.handle()==*owner).and_then(|c|c.full()),_=>None};
    let valid=match action {
        C::Slots(delta)=>matches!(effect.source,Source::None)&&state.phase==Phase::PlayerStart&&state.item_context.as_ref().is_some_and(|items|items.m20_profile()&&(
            delta==3&&state.turn==1&&items.owns_relic(crate::content::RelicId::RunicCapacitor)
            ||delta==1&&items.owns_relic(crate::content::RelicId::Inserter)))
            ||actual.is_some_and(|card|match card.id{CardId::Capacitor=>delta==if card.upgrades>0{3}else{2},CardId::Consume=>delta== -1,_=>false})
            ||matches!(&effect.source,Source::Potion{slot,id:crate::content::PotionId::PotionOfCapacity} if state.item_context.as_ref().is_some_and(|items|
                items.potions().get(*slot).is_some_and(|p|p.is_none()||*p==Some(crate::content::PotionId::PotionOfCapacity))&&delta==if items.owns_relic(crate::content::RelicId::SacredBark){4}else{2})),
        C::Recursion=>actual.is_some_and(|card|card.id==CardId::Recursion),
        C::Fission{evoke}=>actual.is_some_and(|card|card.id==CardId::Fission&&evoke==(card.upgrades>0)),
        C::Lightning{amount}=>state.full_runtime&&(0..=1007).contains(&amount),
        C::OrbEnergy(amount)=>state.full_runtime&&(1..=10).contains(&amount),
        C::ChannelState{orb,..}=>(orb.kind==OrbKind::Dark)==orb.dark_damage.is_some(),
        C::PassiveGroup(OrbPassiveGroup::Dark)=>actual.is_some_and(|card|card.id==CardId::Darkness&&card.upgrades>0),
        C::PassiveGroup(OrbPassiveGroup::All)=>matches!(effect.source,Source::Actor(Actor::Player))&&state.item_context.as_ref().is_some_and(|items|items.owns_relic(crate::content::RelicId::EmotionChip)),
        _=>true,
    };
    if !valid||effect.target!=Some(Actor::Player){return Err(InputError::Invariant("character work requires typed producer and payload".into()))}Ok(())
}

pub(in crate::combat) fn valid_mutable_attack_source(state:&State,source:&Source)->bool{
    matches!(source,Source::Card(owner) if super::super::piles::owned_full_cards(state).any(|card|card.id==CardId::Claw&&super::super::card_state::handle(card.instance)==*owner))
}

pub(in crate::combat) fn validate_power_application(state:&State,effect:&Effect,id:FullPowerId,amount:i32)->Result<(),InputError>{
    let Source::Card(owner)=&effect.source else{return Err(InputError::Invariant("blue power requires actual notified card".into()))};
    let Some(card)=state.in_play.as_ref().filter(|c|c.handle()==*owner).and_then(|c|c.full())else{return Err(InputError::Invariant("blue power in-play source".into()))};
    let up=card.upgrades>0;
    let expected=match (id.0,card.id){
        ("buffer",CardId::Buffer)=>if up{2}else{1},
        ("repair",CardId::SelfRepair)=>if up{10}else{7},
        ("draw",CardId::MachineLearning)|("hello_world",CardId::HelloWorld)|("creative_ai",CardId::CreativeAI)|("equilibrium",CardId::Equilibrium)|("rebound",CardId::Rebound)=>1,
        ("energized_blue",CardId::ChargeBattery)|("bias",CardId::BiasedCognition)|("storm",CardId::Storm)|("echo_form",CardId::EchoForm)|("electrodynamics",CardId::Electrodynamics)=>1,
        ("loop",CardId::Loop)|("static_discharge",CardId::StaticDischarge)|("heatsinks",CardId::Heatsinks)|("amplify",CardId::Amplify)=>if up{2}else{1},
        ("lock_on",CardId::Bullseye)=>if up{3}else{2},
        _=>return Err(InputError::Invariant("blue power producer identity".into())),
    };
    let target=if id.0=="lock_on"{matches!(effect.target,Some(Actor::Enemy(_)))}else{effect.target==Some(Actor::Player)};
    if !target||amount!=expected{return Err(InputError::Invariant("blue power exact amount/target".into()))}Ok(())
}

pub(in crate::combat) fn capture_direct(state:&mut State,effects:&mut [Effect])->Result<(),EffectError>{powers::capture(state,effects);direct::capture(state,effects)}
pub(in crate::combat) fn refresh_values(card:&mut LiveCard,frost:u32,lightning:u32,discard:usize){direct::refresh_values(card,frost,lightning,discard)}

pub(in crate::combat) fn capped_energy_source(state:&State,source:&Source,amount:i32)->bool {
    // Preserve the existing variable-gain producers and frozen old profiles.
    if matches!(source,Source::Card(owner) if super::super::piles::owned_full_cards(state).any(|card|
        super::super::card_state::handle(card.instance)==*owner&&matches!(card.id,CardId::Aggregate|CardId::DoubleEnergy|CardId::Recycle))){return true;}
    if state.silent.source_character!=Some(crate::content::Character::Defect)
        ||!state.item_context.as_ref().is_some_and(|items|items.m20_profile()){return false;}
    match source {
        Source::Card(owner)=>state.in_play.as_ref().filter(|card|card.handle()==*owner).and_then(|card|card.full()).is_some_and(|card|match card.id{
            CardId::Turbo=>amount==if card.upgrades>0{3}else{2},CardId::Sunder=>amount==3,_=>false,
        }),
        Source::Potion{slot,id:crate::content::PotionId::EnergyPotion}=>{
            let items=state.item_context.as_ref().unwrap();
            let owner=if let Some(current)=state.current.as_ref(){
                current.source==*source&&current.target==Some(Actor::Player)&&current.kind==EffectKind::GainEnergy(amount)
                    &&items.potions().get(*slot)==Some(&None)
            }else{items.potions().get(*slot)==Some(&Some(crate::content::PotionId::EnergyPotion))};
            owner&&amount==if items.owns_relic(crate::content::RelicId::SacredBark){4}else{2}
        },
        _=>false,
    }
}

pub(in crate::combat) fn power_hook(state:&mut State,hook:Hook,target:Option<&Actor>,work:&mut Vec<Effect>)->Result<(),EffectError>{powers::hook(state,hook,target,work)}
