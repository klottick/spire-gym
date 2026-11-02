use super::*;
use crate::content::{CardId, PotionId, RelicId};
pub(crate) fn gain_gold(state:&mut RunState,amount:u32)->Result<(),RunError>{
    // M27 gold seams (design §6.5): handlers may change the amount before it is added.
    let mut gaining=crate::items::dispatch::RunItemEvent::GoldGaining{amount};
    crate::items::dispatch::run_event(state,&mut gaining)?;
    let crate::items::dispatch::RunItemEvent::GoldGaining{amount}=gaining else {unreachable!("gold event kept its kind")};
    state.gold=state.gold.checked_add(amount).ok_or(RunError::Unsupported("gold overflow"))?;
    crate::items::dispatch::run_event(state,&mut crate::items::dispatch::RunItemEvent::GoldGained{amount})
}
pub(super) fn supported(id:CardId)->bool {matches!(id,CardId::Strike|CardId::Defend|CardId::Bash|CardId::Clothesline
    |CardId::IronWave|CardId::PommelStrike|CardId::ShrugItOff|CardId::Anger|CardId::Bloodletting|CardId::BurningPact|CardId::Disarm)}
pub(super) fn supported_in(state:&RunState,id:CardId)->bool{if super::rooms::is_m25(state){return super::M25_WATCHER_HEART_CARDS.contains(&id)}if super::rooms::is_m24(state){return super::M24_WATCHER_ACT3_CARDS.contains(&id)}if super::rooms::uses_watcher_content(state){return super::M22_WATCHER_ACT1_CARDS.contains(&id)}if super::rooms::is_m20(state){return id==CardId::Parasite||crate::content::defect::supports_m20_permanent(id)}if super::rooms::is_m19(state){return super::M19_DEFECT_ACT3_CARDS.contains(&id)&&(id==CardId::Parasite||crate::content::defect::supports_permanent(id))}if super::rooms::is_m18(state){return super::M18_DEFECT_ACT2_CARDS.contains(&id)&&crate::content::defect::supports_permanent(id)}if super::rooms::is_m17(state){return super::M17_DEFECT_ACT1_CARDS.contains(&id)}if state.authored_combat&&state.character==crate::content::Character::Defect{return crate::content::defect::supports_permanent(id)}if super::rooms::is_m15(state){return super::M15_SILENT_HEART_CARDS.contains(&id)}if super::rooms::is_m14(state){return super::M14_SILENT_ACT3_CARDS.contains(&id)}if super::rooms::is_m12(state)||super::rooms::is_m13(state){return super::M12_SILENT_ACT1_CARDS.contains(&id)}supported(id)||(super::rooms::is_m10(state)&&[crate::content::CardRarity::Basic,crate::content::CardRarity::Common,crate::content::CardRarity::Uncommon,crate::content::CardRarity::Rare].into_iter().any(|rarity|super::rooms::m10_content::red_candidates(rarity).contains(&id)))||(super::rooms::uses_captured_act1(state)&&matches!(id,CardId::Cleave|CardId::TwinStrike|CardId::Thunderclap|CardId::Bludgeon|CardId::Impervious|CardId::Offering))||(super::rooms::uses_captured_act3(state)&&state.act==3&&id==CardId::Inflame)}
/// Complete-item obtain owners (rewards, shop, relic equips): a negated obtain is `Ok(None)`.
pub(crate) fn obtain_card_or_negated(state:&mut RunState,id:CardId,upgrades:u32)->Result<Option<u64>,RunError>{
    if state.card_admission!=CardAdmission::Complete{return obtain_card(state,id,upgrades).map(Some)}
    if !crate::content::complete::supports_permanent(id)||!super::training::complete_upgrades_valid(id,upgrades){return Err(RunError::InvalidAction)}
    push_card_or_negated(state,id,upgrades,super::training::complete_obtain_misc(id),false)
}
pub(crate) fn obtain_card(state:&mut RunState,id:CardId,upgrades:u32)->Result<u64,RunError>{
    if state.card_admission==CardAdmission::Complete{
        // Design §8.1: any complete permanent card; Searing Blow any count, curses 0, others 0 or 1.
        if !crate::content::complete::supports_permanent(id)||!super::training::complete_upgrades_valid(id,upgrades){return Err(RunError::InvalidAction)}
        return push_card(state,id,upgrades,super::training::complete_obtain_misc(id));
    }
    if (super::rooms::is_m20(state)||super::rooms::is_m24(state)||super::rooms::is_m25(state))&&id==CardId::Parasite&&upgrades!=0{return Err(RunError::InvalidAction)}
    if !supported_in(state,id)||upgrades>1{return Err(RunError::InvalidAction)}
    push_card(state,id,upgrades,if id==CardId::GeneticAlgorithm{1}else{0})
}
/// The one physical-instance allocator for obtain, duplicate and note retrieval.
fn push_card(state:&mut RunState,id:CardId,upgrades:u32,misc:i32)->Result<u64,RunError>{
    push_card_or_negated(state,id,upgrades,misc,false)?.ok_or(RunError::Unsupported("a negated card obtain needs an *_or_negated owner"))
}
/// Design §6.5: a `CardObtaining` handler (Omamori, complete items only) may negate the obtain;
/// the card is then not created and `None` is returned.
fn push_card_or_negated(state:&mut RunState,id:CardId,upgrades:u32,misc:i32,direct:bool)->Result<Option<u64>,RunError>{
    if state.deck.len()>=999{return Err(RunError::InvalidAction)}
    // M27 card-obtain seam (design §6.5, H0 §2): every obtain path reaches this allocator.
    let mut obtaining=crate::items::dispatch::RunItemEvent::CardObtaining{id,upgrades,negated:false,direct};
    crate::items::dispatch::run_event(state,&mut obtaining)?;
    let crate::items::dispatch::RunItemEvent::CardObtaining{upgrades,negated,..}=obtaining else {unreachable!("obtain event kept its kind")};
    if negated{return Ok(None)}
    let instance=state.next_instance;let next=instance.checked_add(1).ok_or(RunError::Unsupported("permanent identity exhausted"))?;
    if instance==0||state.deck.iter().any(|c|c.instance==instance){return Err(RunError::InvalidAction)}
    state.deck.push(CardInstance{misc,id,instance,upgrades});state.next_instance=next;
    crate::items::dispatch::run_event(state,&mut crate::items::dispatch::RunItemEvent::CardObtained{id,instance})?;Ok(Some(instance))
}
pub(crate) fn remove_card(state:&mut RunState,id:u64)->Result<CardInstance,RunError>{
    let at=state.deck.iter().position(|c|c.instance==id).ok_or(RunError::InvalidAction)?;
    // Design §8.2: CurseOfTheBell, Necronomicurse and Ascender's Bane are never removed.
    if !removable(state.deck[at].id){return Err(RunError::InvalidAction)}
    let parasite=state.deck[at].id==CardId::Parasite;
    if parasite&&(state.combat.is_some()||state.max_hp==0||state.max_hp>999||state.hp>state.max_hp){return Err(RunError::InvalidAction)}
    let card=state.deck.remove(at);
    if parasite{decrease_max_hp(state,3);}
    crate::items::dispatch::run_event(state,&mut crate::items::dispatch::RunItemEvent::CardRemoved)?;Ok(card)
}
pub(crate) fn upgrade_card(state:&mut RunState,id:u64)->Result<(),RunError>{
    let at=state.deck.iter().position(|c|c.instance==id&&can_upgrade(state,c)).ok_or(RunError::InvalidAction)?;
    let original=&state.deck[at];
    let mut live=crate::combat::permanent_live_card_with_misc(original.id,crate::combat::CardInstanceId(original.instance),original.upgrades,original.misc);
    crate::combat::cards::ironclad::rules::upgrade(&mut live)?;
    state.deck[at].upgrades=live.upgrades;Ok(())
}
pub(crate) fn can_upgrade(state:&RunState,card:&CardInstance)->bool{
    if state.card_admission==CardAdmission::Complete{
        return crate::content::complete::supports_permanent(card.id)&&crate::combat::cards::ironclad::rules::can_upgrade(card.id,card.upgrades)
    }
    (supported_in(state,card.id)||!super::rooms::uses_watcher_content(state)&&!super::rooms::is_m20(state)&&!super::rooms::uses_silent_content(state)&&matches!(card.id,CardId::Armaments|CardId::SearingBlow))
        &&crate::combat::cards::ironclad::rules::can_upgrade(card.id,card.upgrades)
}
pub(in crate::run) fn removable(id:CardId)->bool{!crate::content::complete::supports_permanent(id)||crate::content::complete::is_removable(id)}
#[cfg_attr(not(test),allow(dead_code))] // Run callers are M27/M28 (design §7.2, §8.3-8.5).
fn admits(state:&RunState,id:CardId)->bool{
    if state.card_admission==CardAdmission::Complete{crate::content::complete::supports_permanent(id)}else{supported_in(state,id)}
}
#[cfg_attr(not(test),allow(dead_code))] // Run callers are M27/M28 (design §7.2, §8.3-8.5).
fn pool_error(_:crate::content::RegistryError)->RunError{RunError::Unsupported("card pool is incomplete")}
#[cfg_attr(not(test),allow(dead_code))] // Run callers are M27/M28 (design §7.2, §8.3-8.5).
/// Design §8.3: remove a removable card (Parasite's loss applies) and obtain one fresh card
/// drawn uniformly from `Transform(character, input)`. Preflights before any mutation.
pub(crate) fn transform_card(state:&mut RunState,instance:u64,auto_upgrade:bool,key:&[u64])->Result<u64,RunError>{
    let mut staged=state.clone();
    let created=transform_card_or_negated(&mut staged,instance,auto_upgrade,key)?
        .ok_or(RunError::Unsupported("a negated card obtain needs an *_or_negated owner"))?;
    *state=staged;Ok(created)
}
/// As `transform_card`; the removal stands when the fresh card's obtain is negated.
pub(crate) fn transform_card_or_negated(state:&mut RunState,instance:u64,auto_upgrade:bool,key:&[u64])->Result<Option<u64>,RunError>{
    let input=state.deck.iter().find(|c|c.instance==instance).ok_or(RunError::InvalidAction)?.id;
    if !removable(input)||!admits(state,input){return Err(RunError::InvalidAction)}
    let support=crate::content::pools::cards(crate::content::pools::CardPool::Transform{character:state.character,input}).map_err(pool_error)?;
    let mut staged=state.clone();
    let index=staged.rng.bounded_scoped_checked(super::RunRngScope::M26CardTransform,key,support.len() as u64)?;
    let output=support[index];
    let upgrades=u32::from(auto_upgrade&&crate::combat::cards::ironclad::rules::can_upgrade(output,0));
    remove_card(&mut staged,instance)?;
    if !admits(&staged,output){return Err(RunError::InvalidAction)}
    let created=push_card_or_negated(&mut staged,output,upgrades,super::training::complete_obtain_misc(output),false)?;
    *state=staged;Ok(created)
}
#[cfg_attr(not(test),allow(dead_code))] // Run callers are M27/M28 (design §7.2, §8.3-8.5).
/// Design §8.4: a new physical instance with the same id, upgrades and `misc`.
pub(crate) fn duplicate_card(state:&mut RunState,instance:u64)->Result<u64,RunError>{
    let mut staged=state.clone();
    let created=duplicate_card_or_negated(&mut staged,instance)?.ok_or(RunError::Unsupported("a negated card obtain needs an *_or_negated owner"))?;
    *state=staged;Ok(created)
}
pub(crate) fn duplicate_card_or_negated(state:&mut RunState,instance:u64)->Result<Option<u64>,RunError>{
    let card=state.deck.iter().find(|c|c.instance==instance).ok_or(RunError::InvalidAction)?.clone();
    if !admits(state,card.id){return Err(RunError::InvalidAction)}
    let mut staged=state.clone();let created=push_card_or_negated(&mut staged,card.id,card.upgrades,card.misc,false)?;*state=staged;Ok(created)
}
#[cfg_attr(not(test),allow(dead_code))] // Run callers are M27/M28 (design §7.2, §8.3-8.5).
/// Design §8.5: a stored card returns fresh, its upgrades applied through the shared upgrade
/// rule (a stored Searing Blow keeps its count) and base `misc`.
pub(super) fn obtain_stored_card(state:&mut RunState,id:CardId,upgrades:u32)->Result<u64,RunError>{
    let storable=crate::content::pools::cards(crate::content::pools::CardPool::NoteForYourself).map_err(pool_error)?;
    if !storable.contains(&id)||!admits(state,id){return Err(RunError::InvalidAction)}
    let misc=super::training::complete_obtain_misc(id);
    let mut live=crate::combat::permanent_live_card_with_misc(id,crate::combat::CardInstanceId(1),0,misc);
    for _ in 0..upgrades{if !crate::combat::cards::ironclad::rules::can_upgrade(id,live.upgrades){return Err(RunError::InvalidAction)}crate::combat::cards::ironclad::rules::upgrade(&mut live)?;}
    // Note's take obtains directly: Omamori never negates it (event-neow research §6, "Note direct obtain").
    let mut staged=state.clone();
    let created=push_card_or_negated(&mut staged,id,live.upgrades,misc,true)?.ok_or(RunError::InvalidAction)?;
    *state=staged;Ok(created)
}
#[cfg_attr(not(test),allow(dead_code))] // Run callers are M27/M28 (design §7.2, §8.3-8.5).
/// Design §7.3: one uniform draw from a Run pool (`run.m26.card.curse` for the curse pool,
/// otherwise `run.m26.card.offer`); RNG commits only on success.
pub(super) fn draw_card(state:&mut RunState,pool:crate::content::pools::CardPool,key:&[u64])->Result<CardId,RunError>{
    Ok(draw_distinct(state,pool,1,key)?[0])
}
#[cfg_attr(not(test),allow(dead_code))] // Run callers are M27/M28 (design §7.2, §8.3-8.5).
/// `count` distinct cards from one pool through the shared staged sampler.
pub(super) fn draw_distinct(state:&mut RunState,pool:crate::content::pools::CardPool,count:usize,key:&[u64])->Result<Vec<CardId>,RunError>{
    if count==0{return Err(RunError::InvalidAction)}
    let family=crate::content::pools::cards(pool).map_err(pool_error)?;
    let scope=if pool==crate::content::pools::CardPool::Curse{super::RunRngScope::M26CardCurse}else{super::RunRngScope::M26CardOffer};
    let slots=vec![family.as_slice();count];
    super::rooms::m15_content::sample_capture_row_for(&mut state.rng,key,&slots,scope,"card pool is incomplete","card pool has too few distinct cards")
}
pub(crate) fn heal(state:&mut RunState,amount:u32)->Result<(),RunError>{
    if state.max_hp==0||state.max_hp>999||state.hp>state.max_hp{return Err(RunError::InvalidAction)}
    let amount=i32::try_from(amount).map_err(|_|RunError::InvalidAction)?;
    let heal=crate::items::ordinary_heal_amount(&state.relics,amount).max(0) as u32;
    let before=state.hp;
    if state.hp>0{state.hp=state.hp.saturating_add(heal).min(state.max_hp)}
    crate::combat::observe::run_heal(state,before);Ok(())
}
/// An out-of-combat HP change's source: HP_LOSS, or damage with or without an owner (events' own
/// damage has none).
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) enum OutOfCombatLoss { HpLoss, Damage{owned:bool} }

/// The one out-of-combat player damage / HP-loss law (PM source check m28c Q4 and its Torii
/// correction; events and Neow use it): Torii makes owned non-HP_LOSS damage of 2–5 into 1; then
/// Tungsten Rod takes 1 from a positive loss; below 1 HP a Fairy in a Bottle rescues first (its
/// stored potency, 30 or Sacred Bark's 60, percent of max HP), else an unused Lizard Tail (half max
/// HP); Mark of the Bloom blocks both. Buffer is a combat power and never exists here. Returns
/// whether the player lives; the caller settles a death.
pub(crate) fn lose_hp(state:&mut RunState,amount:u32,kind:OutOfCombatLoss)->bool{
    let amount=if kind==(OutOfCombatLoss::Damage{owned:true})&&(2..=5).contains(&amount)&&state.relics.contains(&RelicId::Torii){1}else{amount};
    let amount=if amount>0&&state.relics.contains(&RelicId::TungstenRod){amount-1}else{amount};
    state.hp=state.hp.saturating_sub(amount);
    if state.hp>0{return true}
    if state.relics.contains(&RelicId::MarkOfBloom){return false}
    if let Some(slot)=state.potions.iter().position(|potion|*potion==Some(PotionId::FairyPotion)){
        let stored=state.potion_potencies.get(slot).copied().flatten().map(|(_,potency)|potency);
        let percent=stored.unwrap_or(if state.relics.contains(&RelicId::SacredBark){60}else{30});
        state.potions[slot]=None;
        if let Some(potency)=state.potion_potencies.get_mut(slot){*potency=None;}
        if !state.potions.iter().any(Option::is_some){state.potion_potencies.clear();}
        state.hp=(((state.max_hp as f32)*(percent as f32/100.0_f32)) as u32).clamp(1,state.max_hp);
        return true
    }
    if state.relics.contains(&RelicId::LizardTail)&&state.relic_counters.get(&RelicId::LizardTail).copied().unwrap_or(0)==0{
        state.relic_counters.insert(RelicId::LizardTail,1);
        state.hp=(state.max_hp/2).max(1);
        return true
    }
    false
}
/// The one max-HP loss owner (M28 design §5.4): max HP falls by `amount` to at least 1 and
/// current HP is clamped to it. Parasite, Neow's drawback and events call it.
pub(crate) fn decrease_max_hp(state:&mut RunState,amount:u32){state.max_hp=state.max_hp.saturating_sub(amount).max(1);state.hp=state.hp.min(state.max_hp);}
pub(crate) fn increase_max_hp(state:&mut RunState,amount:u32)->Result<(),RunError>{
    let max=state.max_hp.checked_add(amount).filter(|m|*m<=999).ok_or(RunError::Unsupported("maximum HP overflow"))?;
    if state.max_hp==0||state.hp>state.max_hp||amount>i32::MAX as u32{return Err(RunError::InvalidAction)}
    state.max_hp=max;heal(state,amount)
}
pub(super) fn acquire_key(state:&mut RunState,key:Key)->Result<CommittedEvent,RunError>{
    let i=super::reward::key_index(key);if state.keys[i]{return Err(RunError::InvalidAction)}
    let event=CommittedEvent::KeyAcquired(key);super::reward::preflight(state,std::slice::from_ref(&event))?;
    state.keys[i]=true;Ok(event)
}
