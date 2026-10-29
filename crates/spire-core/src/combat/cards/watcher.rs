//! Finite Watcher producers on the shared resolver and card owner.
use super::*;

pub(in crate::combat) fn initialize(card:&mut LiveCard) {
    use crate::content::CardId::*;let up=card.upgrades>0;
    card.local.self_retain=matches!(card.id,WindmillStrike|Perseverance|SandsOfTime|Miracle|Smite|Safety)||card.id==Worship&&up;
    if matches!(card.id,Smite|Safety){card.local.exhaust_on_finish=true;}
    if matches!(card.id,Establishment|BattleHymn){card.local.innate=up;}
    match card.id {
        FlurryOfBlows|Weave=>card.local.base_damage=Some(if up{6}else{4}),
        Brilliance=>card.local.base_damage=Some(if up{16}else{12}),
        WindmillStrike=>card.local.base_damage=Some(if up{10}else{7}),
        SandsOfTime=>card.local.base_damage=Some(if up{26}else{20}),
        CarveReality=>card.local.base_damage=Some(if up{10}else{6}),
        CutThroughFate=>card.local.base_damage=Some(if up{9}else{7}),
        JustLucky=>{card.local.base_damage=Some(if up{4}else{3});card.local.base_block=Some(if up{3}else{2});},
        Smite=>card.local.base_damage=Some(if up{16}else{12}),
        Perseverance=>card.local.base_block=Some(if up{7}else{5}),
        DeceiveReality=>card.local.base_block=Some(if up{7}else{4}),
        ThirdEye=>card.local.base_block=Some(if up{9}else{7}),
        Safety=>card.local.base_block=Some(if up{16}else{12}),
        Prostrate=>card.local.base_block=Some(4),
        _=>(),
    }
}

#[derive(Clone,Debug,PartialEq,Eq)]
pub enum WatcherEffect { OpenStanceChoice, PotionEnergy{slot:usize,amount:i32}, CreateRequest{id:crate::content::CardId,count:u32}, Create{template:LiveCard,count:u32,next:u32}, Meditate{count:usize}, RequestEndTurn, Scry{amount:i32}, CommitSelection{selection:super::super::PendingSelection,next:usize}, RetainRestore { cards:Vec<CardHandle>,next:usize }, ReduceCost{card:CardHandle,delta:i32}, EstablishmentDiscount{amount:i32}, GainMantra { amount:i32 }, StanceEnergy { reason:StanceEnergy }, ReturnDiscard { card:CardHandle } }
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum StanceEnergy { CalmExit, DivinityEntry, VioletLotus }
pub(in crate::combat) fn active(state:&State)->bool {
    state.silent.source_character==Some(crate::content::Character::Watcher)
}
fn bottom(state:&mut State,kind:EffectKind) {
    super::super::enemies::act2::bottom(state,Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind});
}
pub(in crate::combat) fn stance_transition(state:&mut State,next:Stance)->Result<bool,EffectError> {
    if !active(state){return Ok(false)}
    let previous=state.character.stance;
    if previous==next||state.player_full_powers.iter().any(|p|p.id.0=="cannot_change_stance"){return Ok(true)}
    for power in state.player_full_powers.clone() {
        match power.id.0 {
            "mental_fortress"=>bottom(state,EffectKind::GainBlock(power.state.amount)),
            "rushdown" if next==Stance::Wrath=>bottom(state,EffectKind::Draw(power.state.amount)),
            _=>(),
        }
    }
    if previous==Stance::Calm {
        if state.item_context.as_ref().is_some_and(|items|items.owns_relic(crate::content::RelicId::VioletLotus)) {
            bottom(state,EffectKind::Watcher(WatcherEffect::StanceEnergy{reason:StanceEnergy::VioletLotus}));
        }
        bottom(state,EffectKind::Watcher(WatcherEffect::StanceEnergy{reason:StanceEnergy::CalmExit}));
    }
    state.character.stance=next;
    if next==Stance::Divinity {bottom(state,EffectKind::Watcher(WatcherEffect::StanceEnergy{reason:StanceEnergy::DivinityEntry}));}
    let returns=state.piles.discard.iter().filter(|card|card.full().is_some_and(|card|card.id==crate::content::CardId::FlurryOfBlows)).map(|card|card.handle()).collect::<Vec<_>>();
    for card in returns{bottom(state,EffectKind::Watcher(WatcherEffect::ReturnDiscard{card}));}
    super::super::effects::refresh_enemy_values(state)?;Ok(true)
}
pub(in crate::combat) fn apply(state:&mut State,effect:&Effect,work:&WatcherEffect)->Result<(),EffectError> {
    validate(state,effect,work)?;
    match work {
        WatcherEffect::OpenStanceChoice=>{
            let Source::Potion{slot,id}=effect.source else{return Err(InputError::Invariant("stance choice lacks potion parent".into()).into())};
            if id!=crate::content::PotionId::StancePotion||state.pending_selection.is_some()||state.silent.card_offer.is_some()||state.watcher.stance_choice.is_some()||state.in_play.is_some()
                ||!state.item_context.as_ref().is_some_and(|i|i.consumed_potion_potency(slot,id)==Some(0)) {return Err(InputError::Invariant("stance potion must be consumed once before choice".into()).into())}
            state.watcher.stance_choice=Some(PendingStanceChoice{parent:SelectionParent::Potion{slot,id},options:vec![Stance::Wrath,Stance::Calm],resume_phase:state.phase});state.phase=Phase::Selection;
        },
        WatcherEffect::PotionEnergy{amount,..}=>super::super::effects::gain_energy(state,*amount,true)?,
        WatcherEffect::CreateRequest{id,count}=>{
            let kind=create_action(state,*id,*count)?;state.pending.push_front(Effect{source:effect.source.clone(),target:Some(Actor::Player),kind});
        },
        WatcherEffect::Create{template,count,next}=>deliver(state,effect,template,*count,*next)?,
        WatcherEffect::Meditate{count}=>meditate(state,effect,*count)?,
        WatcherEffect::RequestEndTurn=>{
            // Keep the shared Decision checkpoint alive until this physical
            // parent's FinishCard (including Time Warp) has completed. Reuse
            // the same queued request; a forced Time Warp owns the single end.
            if let Some(finish)=state.pending.iter().position(|e|e.kind==EffectKind::FinishCard&&e.source==effect.source){
                state.pending.insert(finish+1,effect.clone());
            }else if state.pending_time_warp.is_none(){
                let marker=state.pending.iter_mut().find(|e|e.kind==EffectKind::Decision).ok_or_else(||InputError::Invariant("Meditate lost its parent finish/decision checkpoint".into()))?;
                marker.kind=EffectKind::EndPlayerTurn;
            }
        },
        WatcherEffect::Scry{amount}=>scry(state,effect,*amount)?,
        WatcherEffect::CommitSelection{selection,next}=>commit_selection(state,effect,selection,*next)?,
        WatcherEffect::RetainRestore{cards,next}=>restore(state,effect,cards,*next)?,
        WatcherEffect::ReduceCost{card,delta}=>{
            let mut result=Ok(());super::super::piles::for_each_owned_card_mut(state,|live|{if live.instance.0.to_string()==card.as_str(){result=super::super::card_state::modify_cost_for_combat(live,*delta);}});result?;
        },
        WatcherEffect::EstablishmentDiscount{amount}=>{for owned in &mut state.piles.hand{if let Some(card)=owned.full_mut().filter(|c|c.local.retain||c.local.self_retain){super::super::card_state::modify_cost_for_combat(card,amount.wrapping_neg())?;}}},
        WatcherEffect::GainMantra{amount}=>{
            state.character.mantra_gained=state.character.mantra_gained.checked_add(*amount).ok_or_else(||InputError::Unsupported("Watcher Mantra history int32 exhausted".into()))?;
            bottom(state,EffectKind::ApplyFullPower{id:FullPowerId("mantra"),amount:*amount});
        },
        WatcherEffect::StanceEnergy{reason}=>{
            let amount=match reason{StanceEnergy::CalmExit=>2,StanceEnergy::DivinityEntry=>3,StanceEnergy::VioletLotus=>1};
            super::super::effects::gain_energy(state,amount,true)?;
        },
        WatcherEffect::ReturnDiscard{card}=>{
            super::super::piles::retrieve_ordinary_from_discard(state,card)?;
            super::super::effects::refresh_enemy_values(state)?;
        },
    }Ok(())
}
pub(in crate::combat) fn validate(state:&State,effect:&Effect,work:&WatcherEffect)->Result<(),InputError> {
    if !active(state)||effect.target!=Some(Actor::Player){return Err(InputError::Invariant("Watcher effect lacks its character/target owner".into()))}
    let invalid=||InputError::Invariant("Watcher payload lost its owned source/cursor/template".into());
    let potion_capture=|slot:usize,id:crate::content::PotionId,amount:i32|state.item_context.as_ref().is_some_and(|items|items.m21_profile()&&
        (items.potions().get(slot)==Some(&Some(id))&&items.potion_potency(slot)==Some(amount)||items.consumed_potion_potency(slot,id)==Some(amount)));
    match work{
        WatcherEffect::OpenStanceChoice=>if !matches!(effect.source,Source::Potion{slot,id:crate::content::PotionId::StancePotion} if potion_capture(slot,crate::content::PotionId::StancePotion,0)){return Err(invalid())},
        WatcherEffect::CreateRequest{id,count}=>if *id!=crate::content::CardId::Miracle||!matches!(effect.source,Source::Potion{slot,id:crate::content::PotionId::BottledMiracle} if i32::try_from(*count).is_ok_and(|n|potion_capture(slot,crate::content::PotionId::BottledMiracle,n))){return Err(invalid())},
        WatcherEffect::Create{template,count,next}=>{
            if next>count||*count>i32::MAX as u32||!crate::content::watcher::M21_WATCHER_GENERATED.contains(&template.id)||template.instance!=CardInstanceId(0)||template.origin!=CombatCardRef::CombatLocal(0)||template.upgrades>1||!template.local.m21{return Err(invalid())}
        },
        WatcherEffect::CommitSelection{selection,next}=>{
            let unique=selection.selected.iter().collect::<std::collections::BTreeSet<_>>();
            if !modal(selection.operation)||*next>selection.selected.len()||unique.len()!=selection.selected.len()||selection.selected.iter().any(|id|!selection.candidates.contains(id)||!state.card_ids_issued.contains(id)){return Err(invalid())}
        },
        WatcherEffect::RetainRestore{cards,next}=>if *next>cards.len()||cards.iter().collect::<std::collections::BTreeSet<_>>().len()!=cards.len()||cards.iter().any(|id|!state.card_ids_issued.contains(id)){return Err(invalid())},
        WatcherEffect::StanceEnergy{..}=>if effect.source!=Source::Actor(Actor::Player){return Err(invalid())},
        WatcherEffect::EstablishmentDiscount{amount}=>if *amount<=0||effect.source!=Source::Actor(Actor::Player){return Err(invalid())},
        WatcherEffect::ReduceCost{card,delta}=>if *delta!= -1||!state.card_ids_issued.contains(card){return Err(invalid())},
        WatcherEffect::ReturnDiscard{card}=>if !state.card_ids_issued.contains(card){return Err(invalid())},
        WatcherEffect::Scry{amount}|WatcherEffect::GainMantra{amount}=>if *amount<0{return Err(invalid())},
        WatcherEffect::Meditate{count}=>if !(1..=2).contains(count){return Err(invalid())},
        _=>(),
    }
    if let WatcherEffect::PotionEnergy{slot,amount}=work{
        let valid=matches!(effect.source,Source::Potion{slot:source_slot,id:crate::content::PotionId::EnergyPotion} if source_slot==*slot)
            &&state.item_context.as_ref().is_some_and(|items|items.m21_profile()&&
                (items.potions().get(*slot)==Some(&Some(crate::content::PotionId::EnergyPotion))&&items.potion_potency(*slot)==Some(*amount)
                ||items.consumed_potion_potency(*slot,crate::content::PotionId::EnergyPotion)==Some(*amount)));
        if !valid{return Err(InputError::Invariant("Watcher potion energy requires its actual captured slot/potency".into()))}
    }
    if matches!(work,WatcherEffect::StanceEnergy{reason:StanceEnergy::VioletLotus})&&!state.item_context.as_ref().is_some_and(|items|items.owns_relic(crate::content::RelicId::VioletLotus)){
        return Err(InputError::Invariant("Violet Lotus energy lacks equipped relic".into()))
    }Ok(())
}

pub(in crate::combat) fn starting_cost(id:crate::content::CardId,upgrades:u32)->Option<i32>{
    use crate::content::CardId::*;Some(match id{
        FlurryOfBlows|Prostrate|JustLucky|Weave=>0,
        Worship|WindmillStrike=>2,SandsOfTime=>4,
        MasterReality|Rushdown=>if upgrades>0{0}else{1},
        Brilliance|Devotion|ThirdEye|CutThroughFate|Foresight|Nirvana|Perseverance|Establishment|Meditate|CarveReality|DeceiveReality|BattleHymn|MentalFortress|LikeWater|Smite|Safety=>1,
        _=>return None,
    })
}
pub(in crate::combat) fn payload(state:&State,card:&LiveCard,target:Option<u64>)->Result<Option<(i32,Vec<(Option<PayloadTarget>,EffectKind)>)>,crate::run::RunError>{
    use crate::content::CardId::*;
    let Some(cost)=starting_cost(card.id,card.upgrades)else{return Ok(None)};
    let up=card.upgrades>0;
    let power=match card.id{
        Devotion=>Some(("devotion",if up{3}else{2})),Foresight=>Some(("foresight",if up{4}else{3})),
        Nirvana=>Some(("nirvana",if up{4}else{3})),Establishment=>Some(("establishment",1)),
        MasterReality=>Some(("master_reality",-1)),BattleHymn=>Some(("battle_hymn",1)),
        MentalFortress=>Some(("mental_fortress",if up{6}else{4})),Rushdown=>Some(("rushdown",2)),LikeWater=>Some(("like_water",if up{7}else{5})),_=>None,
    };
    if let Some((id,amount))=power{
        if target.is_some(){return Err(crate::run::RunError::InvalidAction)}
        return Ok(Some((cost,vec![(Some(PayloadTarget::Player),EffectKind::ApplyFullPower{id:FullPowerId(id),amount})])))
    }
    let effects=match card.id{
        FlurryOfBlows|Weave|Brilliance|WindmillStrike|SandsOfTime|Smite=>{
            let target=target.ok_or(crate::run::RunError::InvalidAction)?;
            let mut base=card.local.base_damage.ok_or(crate::run::RunError::InvalidAction)?;
            if card.id==Brilliance{base=base.wrapping_add(state.character.mantra_gained);}
            vec![(Some(PayloadTarget::SelectedEnemy),EffectKind::Damage(DamagePayload::Fixed(super::modified_mutable_damage(state,target,base)?)))]
        },
        Meditate=>{
            if target.is_some(){return Err(crate::run::RunError::InvalidAction)}
            vec![(Some(PayloadTarget::Player),EffectKind::Watcher(WatcherEffect::Meditate{count:if up{2}else{1}})),
                (Some(PayloadTarget::Player),EffectKind::Character(CharacterEffect::Stance(Stance::Calm))),
                (Some(PayloadTarget::Player),EffectKind::Watcher(WatcherEffect::RequestEndTurn))]
        },
        CarveReality|DeceiveReality=>{
            let mut effects=if card.id==CarveReality{let target=target.ok_or(crate::run::RunError::InvalidAction)?;vec![(Some(PayloadTarget::SelectedEnemy),EffectKind::Damage(DamagePayload::Fixed(super::modified_mutable_damage(state,target,card.local.base_damage.unwrap())?)))]}
                else{if target.is_some(){return Err(crate::run::RunError::InvalidAction)}vec![(Some(PayloadTarget::Player),EffectKind::GainBlock(super::modified_block(state,card.local.base_block.unwrap())?))]};
            effects.push((Some(PayloadTarget::Player),create_action(state,if card.id==CarveReality{Smite}else{Safety},1).map_err(|_|crate::run::RunError::InvalidAction)?));effects
        },
        ThirdEye=>{
            if target.is_some(){return Err(crate::run::RunError::InvalidAction)}
            let bonus=if state.item_context.as_ref().is_some_and(|i|i.owns_relic(crate::content::RelicId::GoldenEye)){2}else{0};
            vec![(Some(PayloadTarget::Player),EffectKind::GainBlock(super::modified_block(state,if up{9}else{7})?)),
                (Some(PayloadTarget::Player),EffectKind::Watcher(WatcherEffect::Scry{amount:if up{5+bonus}else{3+bonus}}))]
        },
        CutThroughFate|JustLucky=>{
            let selected=target.ok_or(crate::run::RunError::InvalidAction)?;
            let bonus=if state.item_context.as_ref().is_some_and(|i|i.owns_relic(crate::content::RelicId::GoldenEye)){2}else{0};
            let hit=(Some(PayloadTarget::SelectedEnemy),EffectKind::Damage(DamagePayload::Fixed(super::modified_mutable_damage(state,selected,card.local.base_damage.unwrap())?)));
            let scry=(Some(PayloadTarget::Player),EffectKind::Watcher(WatcherEffect::Scry{amount:bonus+if card.id==CutThroughFate{if up{3}else{2}}else if up{2}else{1}}));
            if card.id==CutThroughFate{vec![hit,scry,(Some(PayloadTarget::Player),EffectKind::Draw(1))]}
            else{vec![scry,(Some(PayloadTarget::Player),EffectKind::GainBlock(super::modified_block(state,if up{3}else{2})?)),hit]}
        },
        Prostrate|Worship=>{
            if target.is_some(){return Err(crate::run::RunError::InvalidAction)}
            let mut effects=vec![(Some(PayloadTarget::Player),EffectKind::Watcher(WatcherEffect::GainMantra{amount:if card.id==Worship{5}else if up{3}else{2}}))];
            if card.id==Prostrate{effects.push((Some(PayloadTarget::Player),EffectKind::GainBlock(super::modified_block(state,4)?)));}effects
        },
        Perseverance|Safety=>{
            if target.is_some(){return Err(crate::run::RunError::InvalidAction)}
            vec![(Some(PayloadTarget::Player),EffectKind::GainBlock(super::modified_mutable_block(state,card.local.base_block.ok_or(crate::run::RunError::InvalidAction)?)?))]
        },
        _=>return Ok(None),
    };Ok(Some((cost,effects)))
}

pub(in crate::combat) fn discard_hand(state:&mut State,effect:&Effect)->Result<(),EffectError>{
    if let Some(temporary)=state.temporary_retain.take(){
        for card in &mut state.piles.hand{if temporary.cards.contains(&card.handle()){if let Some(card)=card.full_mut(){card.local.retain=true;}}}
    }
    let mut retained=Vec::new();let mut i=0;
    while i<state.piles.hand.len(){
        if state.piles.hand[i].full().is_some_and(|c|c.local.retain||c.local.self_retain){
            let card=state.piles.hand.remove(i);retained.push(card.handle());state.inactive_cards.push(card);
        }else{i+=1;}
    }
    state.pending.push_front(Effect{source:effect.source.clone(),target:Some(Actor::Player),kind:EffectKind::Watcher(WatcherEffect::RetainRestore{cards:retained,next:0})});
    while let Some(card)=state.piles.hand.last(){let handle=card.handle();super::super::piles::discard_card(state,&handle,true)?;}
    Ok(())
}
pub(in crate::combat) fn restore(state:&mut State,effect:&Effect,cards:&[CardHandle],next:usize)->Result<(),EffectError>{
    if next==cards.len(){return Ok(())}
    let handle=&cards[next];let index=state.inactive_cards.iter().position(|c|c.handle()==*handle).ok_or_else(||InputError::Invariant("retained physical card lost its custody".into()))?;
    let card=state.inactive_cards[index].full_mut().ok_or_else(||InputError::Invariant("retained legacy card".into()))?;
    let retained=card.local.self_retain||card.local.retain;
    let sands=retained&&card.id==crate::content::CardId::SandsOfTime;
    if retained{
        match card.id{
            crate::content::CardId::WindmillStrike=>card.local.base_damage=Some(card.local.base_damage.unwrap().checked_add(if card.upgrades>0{5}else{4}).ok_or_else(||InputError::Unsupported("Watcher retained damage int32 exhausted".into()))?),
            crate::content::CardId::Perseverance=>card.local.base_block=Some(card.local.base_block.unwrap().checked_add(if card.upgrades>0{3}else{2}).ok_or_else(||InputError::Unsupported("Watcher retained Block int32 exhausted".into()))?),
            _=>(),
        }
    }
    if sands{bottom(state,EffectKind::Watcher(WatcherEffect::ReduceCost{card:handle.clone(),delta:-1}));}
    let owned=state.inactive_cards.remove(index);state.piles.hand.push(owned);
    if retained{state.piles.hand.last_mut().unwrap().full_mut().unwrap().local.retain=false;}
    if let Some(current)=state.current.as_mut(){current.kind=EffectKind::Watcher(WatcherEffect::RetainRestore{cards:cards.to_vec(),next:next+1});}
    state.pending.push_front(Effect{source:effect.source.clone(),target:Some(Actor::Player),kind:EffectKind::Watcher(WatcherEffect::RetainRestore{cards:cards.to_vec(),next:next+1})});
    super::super::piles::refresh_hand_values(state);super::super::effects::refresh_enemy_values(state)?;Ok(())
}

#[derive(Clone,Debug,Default,PartialEq,Eq)]
pub struct WatcherState {pub(in crate::combat) resume_phase:Option<Phase>,pub(in crate::combat) stance_choice:Option<PendingStanceChoice>,pub(in crate::combat) resolving_stance_choice:Option<PendingStanceChoice>}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct PendingStanceChoice{pub parent:SelectionParent,pub options:Vec<Stance>,pub resume_phase:Phase}
pub(in crate::combat) fn modal(operation:SelectionOperation)->bool{matches!(operation,SelectionOperation::Scry|SelectionOperation::MeditateReturn)}
pub(in crate::combat) fn notify_scry_discard(state:&mut State){
    let cards=state.piles.discard.iter().filter(|c|c.full().is_some_and(|c|c.id==crate::content::CardId::Weave)).map(|c|c.handle()).collect::<Vec<_>>();
    for card in cards{bottom(state,EffectKind::Watcher(WatcherEffect::ReturnDiscard{card}));}
}
pub(in crate::combat) fn scry(state:&mut State,effect:&Effect,amount:i32)->Result<(),EffectError>{
    if super::super::enemies::act2::combat_victory_ready(state){return Ok(())}
    for power in state.player_full_powers.clone(){if power.id.0=="nirvana"{bottom(state,EffectKind::GainBlock(power.state.amount));}}
    let candidates=state.piles.draw.iter().rev().take(amount.max(0) as usize).map(|c|c.handle()).collect::<Vec<_>>();
    if candidates.is_empty(){return Ok(())}
    let parent=match &effect.source{Source::Card(card)=>SelectionParent::Card(card.clone()),
        Source::Actor(Actor::Player) if state.phase==Phase::PlayerStart=>SelectionParent::Power{turn:state.turn,id:FullPowerId("foresight")},
        _=>return Err(InputError::Invariant("Scry requires a live card or Foresight parent".into()).into())};
    let resume=state.phase;
    if candidates.len()>state.public.known_draw.len(){state.public.known_draw=candidates.clone();}
    super::super::effects::begin_selection_range(state,parent,SelectionSourceZone::Draw,candidates.clone(),0,candidates.len(),SelectionDisposition::Discard,EmptySelection::Continue)?;
    state.pending_selection.as_mut().unwrap().operation=SelectionOperation::Scry;
    state.watcher.resume_phase=Some(resume);
    notify_scry_discard(state);Ok(())
}
pub(in crate::combat) fn validate_selection(state:&State,s:&PendingSelection)->Result<(),InputError>{
    if !modal(s.operation){return Ok(())}
    let valid_parent=match &s.parent {
        SelectionParent::Card(owner)=>state.in_play.as_ref().filter(|c|c.handle()==*owner).and_then(|c|c.full()).is_some_and(|c|match s.operation{
            SelectionOperation::Scry=>matches!(c.id,crate::content::CardId::ThirdEye|crate::content::CardId::CutThroughFate|crate::content::CardId::JustLucky),
            SelectionOperation::MeditateReturn=>c.id==crate::content::CardId::Meditate,_=>false}),
        SelectionParent::Power{turn,id}=>s.operation==SelectionOperation::Scry&&*turn==state.turn&&id.0=="foresight"&&state.in_play.is_none()
            &&state.watcher.resume_phase==Some(Phase::PlayerStart)&&state.player_full_powers.iter().any(|p|p.id==*id&&p.state.amount>0),
        _=>false,
    };
    if !active(state)||!valid_parent||state.watcher.resume_phase.is_none()
        ||s.source_zone!=if s.operation==SelectionOperation::Scry{SelectionSourceZone::Draw}else{SelectionSourceZone::Discard}
        ||s.disposition!=if s.operation==SelectionOperation::Scry{SelectionDisposition::Discard}else{SelectionDisposition::Hand}{
        return Err(InputError::Invariant("Watcher selection lost exact operation/parent/resume ownership".into()))
    }Ok(())
}
pub(in crate::combat) fn commit_selection(state:&mut State,effect:&Effect,s:&PendingSelection,next:usize)->Result<(),EffectError>{
    if next>s.selected.len(){return Err(InputError::Invariant("Watcher selected movement cursor".into()).into())}
    if next==s.selected.len(){
        if s.operation==SelectionOperation::Scry{notify_scry_discard(state);}
        return Ok(())
    }
    let handle=&s.selected[next];
    if s.operation==SelectionOperation::Scry{
        let index=state.piles.draw.iter().position(|card|card.handle()==*handle).ok_or_else(||InputError::Invariant("Scry selected physical card left draw".into()))?;
        let card=state.piles.draw.remove(index);state.piles.discard.push(card);
        state.public.known_draw.retain(|id|id!=handle);state.public.known_draw_bottom.retain(|id|id!=handle);
    }else{
        if state.piles.hand.len()<10{
            let index=state.piles.discard.iter().position(|card|card.handle()==*handle).ok_or_else(||InputError::Invariant("Meditate selected physical card left discard".into()))?;
            let mut card=state.piles.discard.remove(index);card.full_mut().unwrap().local.retain=true;state.piles.hand.push(card);
        }
    }
    // Record committed movement before any subsequent derived-value callback.
    if let Some(current)=state.current.as_mut(){current.kind=EffectKind::Watcher(WatcherEffect::CommitSelection{selection:s.clone(),next:next+1});}
    super::super::piles::refresh_hand_values(state);super::super::effects::refresh_enemy_values(state)?;
    state.pending.push_front(Effect{source:effect.source.clone(),target:Some(Actor::Player),kind:EffectKind::Watcher(WatcherEffect::CommitSelection{selection:s.clone(),next:next+1})});Ok(())
}
pub(in crate::combat) fn upgrade_local(card:&mut LiveCard)->Result<(),crate::run::RunError>{
    if !card.local.m21&&starting_cost(card.id,0).is_none(){return Ok(())}
    let mut base=super::super::card_state::permanent(card.id,CardInstanceId(0),0);
    let mut upgraded=super::super::card_state::permanent(card.id,CardInstanceId(0),1);
    if card.local.m21{initialize_profile(&mut base).map_err(|_|crate::run::RunError::InvalidAction)?;initialize_profile(&mut upgraded).map_err(|_|crate::run::RunError::InvalidAction)?;}
    if let (Some(old),Some(new))=(base.local.base_damage,upgraded.local.base_damage){card.local.base_damage=Some(card.local.base_damage.unwrap_or(old).wrapping_add(new.wrapping_sub(old)));}
    if let (Some(old),Some(new))=(base.local.base_block,upgraded.local.base_block){card.local.base_block=Some(card.local.base_block.unwrap_or(old).wrapping_add(new.wrapping_sub(old)));}
    card.local.self_retain=upgraded.local.self_retain;card.local.innate=upgraded.local.innate;
    if card.local.m21{card.local.magic_number=upgraded.local.magic_number;}
    let old=base.local.mutable_combat_cost.or_else(||starting_cost(card.id,0)).unwrap();let new=upgraded.local.mutable_combat_cost.or_else(||starting_cost(card.id,1)).unwrap();
    if old!=new{super::super::card_state::upgrade_base_cost(card,old,new)?;}
    Ok(())
}

pub(in crate::combat) fn meditate(state:&mut State,effect:&Effect,count:usize)->Result<(),EffectError>{
    let candidates=state.piles.discard.iter().map(|c|c.handle()).collect::<Vec<_>>();
    if candidates.is_empty(){return Ok(())}
    if candidates.len()<=count{
        for card in &mut state.piles.discard{card.full_mut().unwrap().local.retain=true;}
        for card in candidates{super::super::piles::retrieve_ordinary_from_discard(state,&card)?;}
        return Ok(())
    }
    let Source::Card(parent)=&effect.source else{return Err(InputError::Invariant("Meditate has no physical card parent".into()).into())};
    let resume=state.phase;
    super::super::effects::begin_selection_range(state,SelectionParent::Card(parent.clone()),SelectionSourceZone::Discard,candidates,count,count,SelectionDisposition::Hand,EmptySelection::Continue)?;
    state.pending_selection.as_mut().unwrap().operation=SelectionOperation::MeditateReturn;state.watcher.resume_phase=Some(resume);Ok(())
}
pub(in crate::combat) fn construct_mantra(state:&mut State,amount:i32)->Result<EffectKind,InputError>{
    state.character.mantra_gained=state.character.mantra_gained.checked_add(amount).ok_or_else(||InputError::Unsupported("Watcher Mantra history int32 exhausted".into()))?;
    Ok(EffectKind::ApplyFullPower{id:FullPowerId("mantra"),amount})
}
pub(in crate::combat) fn capture(state:&mut State,effects:&mut [Effect])->Result<(),EffectError>{
    if !active(state){return Ok(())}
    for effect in effects{
        if let EffectKind::Watcher(WatcherEffect::GainMantra{amount})=effect.kind{effect.kind=construct_mantra(state,amount)?;}
        if let EffectKind::Watcher(WatcherEffect::CreateRequest{id,count})=effect.kind{effect.kind=create_action(state,id,count)?;}
        if let EffectKind::CreateCard{id:crate::content::CardId::Miracle,destination:CardDestination::Hand,upgrades:0}=effect.kind{
            if effect.source==Source::None&&state.current.as_ref().is_some_and(|e|e.kind==EffectKind::Hook(Hook::CombatStartedPreDraw)){
                effect.kind=create_action(state,crate::content::CardId::Miracle,1)?;
            }
        }
    }Ok(())
}
pub(in crate::combat) fn hook(state:&mut State,hook:Hook,target:Option<&Actor>,work:&mut Vec<Effect>)->Result<(),EffectError>{
    if !active(state)||target!=Some(&Actor::Player){return Ok(())}
    let source=Source::Actor(Actor::Player);
    if hook==Hook::TurnStartedPreDraw{
        if state.character.stance==Stance::Divinity{work.push(Effect{source:source.clone(),target:Some(Actor::Player),kind:EffectKind::Character(CharacterEffect::Stance(Stance::Neutral))});}
        // Equal-priority start callbacks follow the installed owner list.
        // Only Foresight's EmptyDeckShuffle is addToTop; both Scry and
        // MakeTempCardInHand remain addToBot in that same callback order.
        for power in state.player_full_powers.clone(){match power.id.0{
            "foresight"=>{
                if state.piles.draw.is_empty(){work.insert(0,Effect{source:source.clone(),target:Some(Actor::Player),kind:EffectKind::Recycle});}
                let bonus=if state.item_context.as_ref().is_some_and(|i|i.owns_relic(crate::content::RelicId::GoldenEye)){2}else{0};
                work.push(Effect{source:source.clone(),target:Some(Actor::Player),kind:EffectKind::Watcher(WatcherEffect::Scry{amount:power.state.amount.wrapping_add(bonus)})});
            },
            "battle_hymn" if !super::super::enemies::act2::combat_victory_ready(state)=>{
                work.push(Effect{source:source.clone(),target:Some(Actor::Player),kind:create_action(state,crate::content::CardId::Smite,power.state.amount.max(0) as u32)?});
            },
            _=>(),
        }}
        // Source recharge is synchronous after pre-draw callback construction.
        if state.turn>1{state.player.energy=state.player.max_energy;}
        for power in state.player_full_powers.clone(){if power.id.0=="devotion"{
            let kind=if !state.player_full_powers.iter().any(|p|p.id.0=="mantra")&&power.state.amount>=10{
                EffectKind::Character(CharacterEffect::Stance(Stance::Divinity))
            }else{construct_mantra(state,power.state.amount)?};
            let at=state.pending.iter().position(|e|matches!(e.kind,EffectKind::Hook(Hook::TurnStarted)|EffectKind::Decision)).unwrap_or(state.pending.len());
            state.pending.insert(at,Effect{source:source.clone(),target:Some(Actor::Player),kind});
        }}
    }
    if hook==Hook::PlayerPreEndTurnCards&&state.character.stance==Stance::Calm{
        for power in &state.player_full_powers{if power.id.0=="like_water"{work.push(Effect{source:source.clone(),target:Some(Actor::Player),kind:EffectKind::GainBlock(power.state.amount)});}}
    }
    Ok(())
}

/// Explicit profile initialization on the existing physical constructor owner.
/// Old bare factories retain their accepted fallback fields and Debug output.
pub(in crate::combat) fn initialize_profile(card:&mut LiveCard)->Result<(),InputError>{
    use crate::content::CardId::*;
    if !crate::content::watcher::supports_generated(card.id)||card.upgrades>1{return Err(InputError::Unsupported("card outside finite M21 construction profile".into()))}
    initialize(card);let up=card.upgrades>0;card.local.m21=true;
    let cost=match card.id{StrikePurple|DefendPurple|EmptyFist=>1,Eruption=>if up{1}else{2},Vigilance=>2,Miracle=>0,_=>starting_cost(card.id,card.upgrades).unwrap()};
    card.local.mutable_combat_cost=Some(cost);card.local.turn_cost=Some(cost);
    match card.id{StrikePurple=>card.local.base_damage=Some(if up{9}else{6}),DefendPurple=>card.local.base_block=Some(if up{8}else{5}),Eruption=>card.local.base_damage=Some(9),Vigilance=>card.local.base_block=Some(if up{12}else{8}),EmptyFist=>card.local.base_damage=Some(if up{14}else{9}),Miracle=>card.local.exhaust_on_finish=true,_=>()}
    card.local.magic_number=match card.id{
        Prostrate|Devotion=>Some(if up{3}else{2}),Worship=>Some(5),Foresight|Nirvana=>Some(if up{4}else{3}),
        ThirdEye=>Some(if up{5}else{3}),CutThroughFate=>Some(if up{3}else{2}),JustLucky|Meditate=>Some(if up{2}else{1}),
        WindmillStrike|MentalFortress=>Some(if up{if card.id==WindmillStrike{5}else{6}}else{4}),Perseverance=>Some(if up{3}else{2}),
        Rushdown=>Some(2),LikeWater=>Some(if up{7}else{5}),
        Establishment|BattleHymn=>Some(1),_=>None,
    };Ok(())
}

fn installed_master_reality(state:&State)->bool{state.player_full_powers.iter().any(|p|p.id.0=="master_reality")}
pub(in crate::combat) fn create_action(state:&State,id:crate::content::CardId,count:u32)->Result<EffectKind,InputError>{
    if !crate::content::watcher::M21_WATCHER_GENERATED.contains(&id){return Err(InputError::Invariant("fixed Watcher child route kind".into()))}
    let upgrades=u32::from(installed_master_reality(state));
    let mut template=super::super::card_state::generated(id,CardInstanceId(0),upgrades).ok_or_else(||InputError::Unsupported("fixed Watcher child constructor unavailable".into()))?;
    initialize_profile(&mut template)?;
    Ok(EffectKind::Watcher(WatcherEffect::Create{template,count,next:0}))
}
fn deliver(state:&mut State,effect:&Effect,template:&LiveCard,count:u32,next:u32)->Result<(),EffectError>{
    if next>count{return Err(InputError::Invariant("fixed creation cursor exceeds captured count".into()).into())}
    if next==count{return Ok(())}
    let instance=super::super::piles::next_generated_instance(state)?;
    let mut child=template.stat_equivalent_copy(instance);
    // The hand helper upgrades its actual inserted child; the invoked overflow
    // helper upgrades only a separate display copy, so the argument stays early.
    if state.piles.hand.len()<10&&installed_master_reality(state)&&child.upgrades==0{
        upgrade_local(&mut child).map_err(|_|InputError::Invariant("fixed child upgrade failed".into()))?;
        child.upgrades=1;
    }
    super::super::piles::insert_generated(state,child,CardDestination::Hand)?;
    if let Some(current)=state.current.as_mut(){current.kind=EffectKind::Watcher(WatcherEffect::Create{template:template.clone(),count,next:next+1});}
    super::super::effects::refresh_enemy_values(state)?;
    state.pending.push_front(Effect{source:effect.source.clone(),target:Some(Actor::Player),kind:EffectKind::Watcher(WatcherEffect::Create{template:template.clone(),count,next:next+1})});Ok(())
}
pub(in crate::combat) fn public_values(card:&LiveCard)->Option<CardPublicValues>{
    if !card.local.m21&&starting_cost(card.id,card.upgrades).is_none(){return None}
    Some(CardPublicValues{base_damage:card.local.base_damage,base_block:card.local.base_block,magic_number:card.local.magic_number,damage_bonus:None,
        cost_kind:CardCostKind::Fixed,free_play:card.local.free_play||card.local.free_to_play_once,exhaust_on_finish:card.local.exhaust_on_finish,
        ethereal:card.local.ethereal,innate:card.local.innate,temporary_cost:card.local.temporary_cost})
}
pub(in crate::combat) fn card_values(state:&State,card:&LiveCard)->Option<super::super::view::WatcherCardValues>{
    if !active(state){return None}
    let combat=super::base_cost(state,card).and_then(|base|super::super::card_state::combat_price(card,base).ok());
    Some(super::super::view::WatcherCardValues{combat_cost:combat,turn_cost:card.local.turn_cost.or(combat),self_retain:card.local.self_retain,retain:card.local.retain})
}
pub(in crate::combat) fn validate_choice(state:&State)->Result<(),InputError>{
    let choice=state.watcher.stance_choice.as_ref().ok_or_else(||InputError::Invariant("stance choice absent".into()))?;
    let valid=match choice.parent{SelectionParent::Potion{slot,id:crate::content::PotionId::StancePotion}=>state.item_context.as_ref().is_some_and(|i|i.consumed_potion_potency(slot,crate::content::PotionId::StancePotion)==Some(0)),_=>false};
    if !active(state)||!valid||state.phase!=Phase::Selection||state.pending_selection.is_some()||state.silent.card_offer.is_some()||state.in_play.is_some()
        ||choice.options!=[Stance::Wrath,Stance::Calm]||matches!(choice.resume_phase,Phase::Selection|Phase::Terminal){return Err(InputError::Invariant("stance choice exact potion/phase/options owner".into()))}Ok(())
}

/// Miracle reaches the same source addEnergy helper as stance and potion gains.
/// The opt-in binding leaves legacy bare-profile overflow behavior unchanged.
pub(in crate::combat) fn capped_energy_source(state:&State,source:&Source,amount:i32)->bool{
    active(state)&&matches!(source,Source::Card(owner) if state.in_play.as_ref().filter(|c|c.handle()==*owner).and_then(|c|c.full()).is_some_and(|c|c.id==crate::content::CardId::Miracle&&amount==if c.upgrades>0{2}else{1}))
}

pub(in crate::combat) fn power_id(id:FullPowerId)->bool{matches!(id.0,"devotion"|"foresight"|"nirvana"|"establishment"|"master_reality"|"battle_hymn"|"mental_fortress"|"rushdown"|"like_water"|"mantra")}

pub(in crate::combat) fn valid_mutable_attack_source(state:&State,source:&Source)->bool{
    active(state)&&matches!(source,Source::Card(owner) if state.in_play.as_ref().filter(|c|c.handle()==*owner).and_then(|c|c.full()).is_some_and(|c|crate::content::watcher::supports_generated(c.id)&&c.local.base_damage.is_some()))
}
