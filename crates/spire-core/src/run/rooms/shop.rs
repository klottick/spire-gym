use super::*;
use crate::content::{CardColor, CardRarity, CardType, PotionRarity};
pub(in crate::run) fn screen(run:&Run,next:&mut u64)->Result<RoomView,RunError>{let mut rng=run.state.rng.clone();screen_with_rng(run,next,&mut rng)}
pub(in crate::run) fn screen_with_rng(run:&Run,next:&mut u64,rng:&mut RunRng)->Result<RoomView,RunError>{
    let state=&run.state;
    let node=if uses_act4(state){state.map.as_ref().and_then(|map|map.current).ok_or(RunError::InvalidAction)?}else{NodeId(0)};
    let destination=(state.act,state.floor,node);
    screen_with_rng_at(run,next,rng,destination)
}
pub(in crate::run) fn screen_with_rng_at<R:RunEventTarget>(mut run:R,next:&mut u64,rng:&mut RunRng,destination:(u8,u32,NodeId))->Result<RoomView,RunError>{
    if is_m25(&run.as_run().state)&&destination.0==4{
        let rows=super::m25_content::act4_shop_rows(run.as_run(),rng,destination.1,destination.2)?;
        return finish_screen(&mut run,next,rows,rng,destination)
    }
    if uses_watcher_content(&run.as_run().state){
        let room=run.as_run().state.room_loop.as_ref().ok_or(RunError::InvalidAction)?;
        // The destination act's pre-increment visit selects first/later stock.
        let later=if (is_m23(&run.as_run().state)||is_m24(&run.as_run().state)||is_m25(&run.as_run().state))&&destination.0==2{room.act2.as_ref().ok_or(RunError::InvalidAction)?.shop_visits>0}else if (is_m24(&run.as_run().state)||is_m25(&run.as_run().state))&&destination.0==3{room.act3.as_ref().ok_or(RunError::InvalidAction)?.shop_visits>0}else{room.act1.as_ref().ok_or(RunError::InvalidAction)?.shop_visits>0};
        let mut rows=super::m22_content::shop_cards(later).into_iter().zip(super::m22_content::SHOP_CARD_PRICES)
            .map(|(id,price)|(RoomChoiceKind::Card{id,upgrades:0},Some(price))).collect::<Vec<_>>();
        let relic=super::m22_content::SHOP_RELICS[usize::from(later)];
        if !run.as_run().state.relics.contains(&relic){rows.push((RoomChoiceKind::Relic(relic),Some(super::m22_content::SHOP_RELIC_PRICE)));}
        rows.extend([(RoomChoiceKind::Potion(super::m22_content::SHOP_POTIONS[usize::from(later)]),Some(super::m22_content::SHOP_POTION_PRICE)),(RoomChoiceKind::Purge,Some(room.purge_cost)),(RoomChoiceKind::Leave,None)]);
        return finish_screen(&mut run,next,rows,rng,destination)
    }
    if is_m15(&run.as_run().state)||is_m20(&run.as_run().state){
        let (act,floor,node)=destination;let room=run.as_run().state.room_loop.as_ref().ok_or(RunError::InvalidAction)?;
        let visit=match act{1=>room.act1.as_ref().ok_or(RunError::InvalidAction)?.shop_visits,2=>room.act2.as_ref().ok_or(RunError::InvalidAction)?.shop_visits,3=>room.act3.as_ref().ok_or(RunError::InvalidAction)?.shop_visits,4=>0,_=>return Err(RunError::InvalidAction)};
        let cards=if is_m20(&run.as_run().state){super::m20_content::sample_capture_row(rng,&[u64::from(act),u64::from(floor),node.0],&[crate::content::CardRarity::Common,crate::content::CardRarity::Uncommon,crate::content::CardRarity::Rare])?}else{super::m15_content::sample_capture_row(rng,&[u64::from(act),u64::from(floor),node.0],&[crate::content::CardRarity::Common,crate::content::CardRarity::Uncommon,crate::content::CardRarity::Rare])?};
        let mut rows=cards.into_iter().zip([50,75,150]).map(|(id,price)|(RoomChoiceKind::Card{id,upgrades:0},Some(price))).collect::<Vec<_>>();
        let relic=if is_m20(&run.as_run().state){RelicId::RunicCapacitor}else{RelicId::TwistedFunnel};
        if !run.as_run().state.relics.contains(&relic){rows.push((RoomChoiceKind::Relic(relic),Some(150)));}
        rows.extend([(RoomChoiceKind::Potion(if is_m20(&run.as_run().state){super::m20_content::POTION_CYCLE[visit%11]}else{super::m15_content::M15_POTION_CYCLE[visit%6]}),Some(50)),(RoomChoiceKind::Purge,Some(room.purge_cost)),(RoomChoiceKind::Leave,None)]);
        return finish_screen(&mut run,next,rows,rng,destination)
    }
    if is_m10(&run.as_run().state){
        let (act,floor,node)=destination;
        let cards=super::m10_content::sample_capture_row(rng,&[u64::from(act),u64::from(floor),node.0],&[crate::content::CardRarity::Common,crate::content::CardRarity::Uncommon,crate::content::CardRarity::Rare])?;
        let mut rows=cards.into_iter().zip([50,75,150]).map(|(id,price)|(RoomChoiceKind::Card{id,upgrades:0},Some(price))).collect::<Vec<_>>();
        rows.extend([(RoomChoiceKind::Relic(RelicId::Brimstone),Some(150)),
            (RoomChoiceKind::Potion(PotionId::BloodPotion),Some(50)),
            (RoomChoiceKind::Potion(PotionId::Elixir),Some(75)),
            (RoomChoiceKind::Potion(PotionId::HeartOfIron),Some(100)),
            (RoomChoiceKind::Purge,Some(run.as_run().state.room_loop.as_ref().ok_or(RunError::InvalidAction)?.purge_cost)),(RoomChoiceKind::Leave,None)]);
        return finish_screen(&mut run,next,rows,rng,destination)
    }
    if uses_defect_content(&run.as_run().state){
        let room=run.as_run().state.room_loop.as_ref().ok_or(RunError::InvalidAction)?;
        let later=if is_m19(&run.as_run().state)&&run.as_run().state.act==3{room.act3.as_ref().is_some_and(|p|p.shop_visits>0)}else if run.as_run().state.act==2{room.act2.as_ref().is_some_and(|p|p.shop_visits>0)}else{room.act1.as_ref().is_some_and(|p|p.shop_visits>0)};
        let cards=if is_m19(&run.as_run().state)&&run.as_run().state.act==3{super::m19_content::shop_cards(later)}else{super::m17_content::shop_cards(later)};
        let mut rows=cards.into_iter().map(|id|(RoomChoiceKind::Card{id,upgrades:0},Some(75))).collect::<Vec<_>>();
        let relic=if later{RelicId::EmotionChip}else{RelicId::GoldPlatedCables};
        if !run.as_run().state.relics.contains(&relic){rows.push((RoomChoiceKind::Relic(relic),Some(150)));}
        rows.extend([(RoomChoiceKind::Potion(if later{PotionId::PotionOfCapacity}else{PotionId::FocusPotion}),Some(50)),(RoomChoiceKind::Purge,Some(room.purge_cost)),(RoomChoiceKind::Leave,None)]);
        return finish_screen(&mut run,next,rows,rng,destination)
    }
    if uses_silent_content(&run.as_run().state){
        let room=run.as_run().state.room_loop.as_ref().ok_or(RunError::InvalidAction)?;
        let later=if is_m14(&run.as_run().state)&&run.as_run().state.act==3{room.act3.as_ref().is_some_and(|p|p.shop_visits>0)}else if (is_m13(&run.as_run().state)||is_m14(&run.as_run().state))&&run.as_run().state.act==2{room.act2.as_ref().is_some_and(|p|p.shop_visits>0)}else{room.act1.as_ref().is_some_and(|p|p.shop_visits>0)};
        let cards=super::m12_content::shop_cards(later);
        let prices=if later{[50,75,150]}else{[50,75,75]};
        let mut rows=cards.into_iter().zip(prices).map(|(id,price)|(RoomChoiceKind::Card{id,upgrades:0},Some(price))).collect::<Vec<_>>();
        let relic=if later{RelicId::BagOfPreparation}else{RelicId::SneckoSkull};
        if !run.as_run().state.relics.contains(&relic){rows.push((RoomChoiceKind::Relic(relic),Some(150)));}
        rows.extend([(RoomChoiceKind::Potion(if later{PotionId::FirePotion}else{PotionId::DexterityPotion}),Some(50)),
            (RoomChoiceKind::Purge,Some(room.purge_cost)),(RoomChoiceKind::Leave,None)]);
        return finish_screen(&mut run,next,rows,rng,destination)
    }
    if uses_captured_act1(&run.as_run().state){
        let later=(uses_captured_act2(&run.as_run().state)&&run.as_run().state.act==2)||(uses_captured_act3(&run.as_run().state)&&run.as_run().state.act==3)||run.as_run().state.room_loop.as_ref().and_then(|r|r.act1.as_ref()).is_some_and(|p|p.shop_visits>0);
        let (card,price,potion,relic)=if later{(CardId::Clothesline,50,PotionId::StrengthPotion,RelicId::Anchor)}else{(CardId::IronWave,25,PotionId::FirePotion,RelicId::BloodVial)};
        let mut rows=vec![(RoomChoiceKind::Card{id:card,upgrades:0},Some(price)),(RoomChoiceKind::Potion(potion),Some(50))];
        if !run.as_run().state.relics.contains(&relic)||(uses_captured_act3(&run.as_run().state)&&run.as_run().state.act==3){rows.push((RoomChoiceKind::Relic(relic),Some(150)));}
        rows.extend([(RoomChoiceKind::Purge,Some(run.as_run().state.room_loop.as_ref().unwrap().purge_cost)),(RoomChoiceKind::Leave,None)]);
        return finish_screen(&mut run,next,rows,rng,destination)
    }
    let purge=run.as_run().state.room_loop.as_ref().unwrap().purge_cost;
    finish_screen(&mut run,next,vec![(RoomChoiceKind::Card{id:CardId::IronWave,upgrades:0},Some(25)),(RoomChoiceKind::Potion(PotionId::FirePotion),Some(50)),(RoomChoiceKind::Relic(RelicId::BloodVial),Some(150)),(RoomChoiceKind::Purge,Some(purge)),(RoomChoiceKind::Leave,None)],rng,destination)
}

const COMPLETE_SHOP_RELIC_SOURCE_TAG:u64=2;
const COMPLETE_COURIER_RELIC_SOURCE_TAG:u64=3;
const COMPLETE_SHOP_POTION_SOURCE_TAG:u64=4;
const COMPLETE_COURIER_POTION_SOURCE_TAG:u64=5;
const COMPLETE_COURIER_CARD_SOURCE_TAG:u64=6;

fn complete_act4_shop<R:RunEventTarget>(run:&R,destination:(u8,u32,NodeId))->bool{
    run.item_admission()==ItemAdmission::Complete&&run.as_run().state.relic_pools.is_some()
        &&destination.0==4&&destination.1==52&&!is_standard(&run.as_run().state)
}

/// The M28 standard shop's rows (design §6.3) after its stock and sticker prices (sale
/// included) are drawn: the shared M27 shop-entry and discount seams, then the screen.
pub(in crate::run) fn standard_screen(run:&mut Run,rows:Vec<(RoomChoiceKind,Option<u32>)>)->Result<RoomView,RunError>{
    if !is_standard(&run.state){return Err(RunError::InvalidAction)}
    let node=run.state.map.as_ref().and_then(|map|map.current).ok_or(RunError::InvalidAction)?;
    let destination=(run.state.act,run.state.floor,node);
    let mut next=run.next_choice;
    let mut rng=run.state.rng.clone();
    let view=finish_screen(&mut &mut *run,&mut next,rows,&mut rng,destination)?;
    run.next_choice=next;
    Ok(view)
}

fn potion_base_price(potion:PotionId)->Result<u32,RunError>{
    match crate::content::Catalog::standard().potion_rarity(potion).ok_or(RunError::Registry("complete potion rarity missing"))?{
        PotionRarity::Common=>Ok(50),PotionRarity::Uncommon=>Ok(75),PotionRarity::Rare=>Ok(100),
    }
}

fn sample_shop_potion(run:&Run,rng:&mut RunRng,destination:(u8,u32,NodeId),row_index:u64,source_tag:u64)->Result<(PotionId,u32),RunError>{
    let key=[u64::from(destination.0),u64::from(destination.1),destination.2.0,row_index,source_tag];
    let character=run.state.character;
    let potion=crate::items::potion_generation::rarity_first(character,|bound|
        rng.bounded_scoped_checked(crate::run::RunRngScope::M27PotionOffer,&key,bound))?;
    Ok((potion,potion_base_price(potion)?))
}

fn finish_screen<R:RunEventTarget>(run:&mut R,next:&mut u64,mut rows:Vec<(RoomChoiceKind,Option<u32>)>,rng:&mut RunRng,destination:(u8,u32,NodeId))->Result<RoomView,RunError>{
    if complete_act4_shop(run,destination) {
        // Act 4 has one fixed floor-52 shop visit; keep the accepted row shape and replace only its identity.
        let key=[u64::from(destination.0),u64::from(destination.1),destination.2.0,0,COMPLETE_SHOP_RELIC_SOURCE_TAG];
        let relic={
            let mut event_state=run.event_state();
            crate::items::relic_pools::take(event_state.state_mut(),crate::content::RelicTier::Shop,
                crate::items::relic_pools::Selector::Back,crate::items::lifecycle::RelicSource::Shop,&key)?
        };
        rows.retain(|(kind,_)|!matches!(kind,RoomChoiceKind::Relic(_)));
        let insert_at=rows.iter().position(|(kind,_)|matches!(kind,RoomChoiceKind::Purge|RoomChoiceKind::Leave)).unwrap_or(rows.len());
        rows.insert(insert_at,(RoomChoiceKind::Relic(relic),Some(150)));
    }
    if complete_act4_shop(run,destination) {
        let potion_rows=rows.iter().enumerate().filter_map(|(index,(kind,_))|
            matches!(kind,RoomChoiceKind::Potion(_)).then_some(index)).collect::<Vec<_>>();
        for (row_index,position) in potion_rows.into_iter().enumerate(){
            let (potion,price)=sample_shop_potion(run.as_run(),rng,destination,row_index as u64,COMPLETE_SHOP_POTION_SOURCE_TAG)?;
            rows[position]=(RoomChoiceKind::Potion(potion),Some(price));
        }
    }
    if run.item_admission()==ItemAdmission::Complete {
        let mut item_event=crate::items::dispatch::RunItemEvent::ShopEntered;
        let mut event_state=run.event_state();
        // M27 seam: shop entered
        crate::items::dispatch::run_event(event_state.state_mut(),&mut item_event)?;
    }
    if run.item_admission()==ItemAdmission::Complete {
        for (kind,price) in &mut rows {
            let Some(base)=*price else {continue};
            if matches!(kind,RoomChoiceKind::Card{..}|RoomChoiceKind::Potion(_)|RoomChoiceKind::Relic(_)) {
                let mut item_event=crate::items::dispatch::RunItemEvent::ShopPrice{price:base};
                let mut event_state=run.event_state();
                // M27 seam: shop price
                crate::items::dispatch::run_event(event_state.state_mut(),&mut item_event)?;
                let crate::items::dispatch::RunItemEvent::ShopPrice{price:returned}=item_event else {unreachable!("shop-price event kept its kind")};
                *price=Some(returned);
            } else if *kind==RoomChoiceKind::Purge {
                let mut item_event=crate::items::dispatch::RunItemEvent::RemovalPrice{price:base};
                let mut event_state=run.event_state();
                // M27 seam: removal price
                crate::items::dispatch::run_event(event_state.state_mut(),&mut item_event)?;
                let crate::items::dispatch::RunItemEvent::RemovalPrice{price:returned}=item_event else {unreachable!("removal-price event kept its kind")};
                *price=Some(returned);
            }
        }
    }
    // Complete-admission shops preview owned eggs on their card rows (G6, carry-over from M28a;
    // source check m4); frozen profiles keep their accepted rows.
    if run.item_admission()==ItemAdmission::Complete{preview_card_rows(&run.as_run().state,rows.iter_mut().map(|(kind,_)|kind));}
    view(next,RoomViewKind::Shop,rows)
}
fn preview_card_rows<'a>(state:&RunState,rows:impl Iterator<Item=&'a mut RoomChoiceKind>){
    for kind in rows{if let RoomChoiceKind::Card{id,upgrades}=kind{*upgrades=super::standard::rewards::preview_upgrades(state,*id,*upgrades);}}
}
pub(in crate::run) fn choose(run:&mut Run,id:ChoiceId)->Result<RunTransition,RunError>{
    let row=public(run)?.choices.into_iter().find(|c|c.id==id&&c.status==RoomChoiceStatus::Offered).ok_or(RunError::InvalidAction)?;
    if row.kind==RoomChoiceKind::Leave{return finish(run)}
    let price=row.price.ok_or(RunError::InvalidAction)?;
    if row.kind==RoomChoiceKind::Purge{return selection::open(run,id,RoomSelectionOperation::ShopPurge{price})}
    if let RoomChoiceKind::Relic(relic)=row.kind.clone(){
        if run.state.item_admission==ItemAdmission::Complete{
            let mut staged=run.clone();
            let obtained=crate::items::obtain_relic(&mut staged.state,relic,crate::items::lifecycle::RelicSource::Shop)?;
            staged.state.gold-=price;
            staged.state.room_loop.as_mut().unwrap().screen.choices.iter_mut().find(|choice|choice.id==id).ok_or(RunError::InvalidAction)?.status=RoomChoiceStatus::Claimed;
            // M27 seam: shop purchase
            crate::items::dispatch::run_event(&mut staged.state,&mut crate::items::dispatch::RunItemEvent::ShopPurchase)?;
            reprice_after_discount_relic(&mut staged,relic)?;
            // A purchased egg re-previews the shop's card rows (economy research :190-197).
            let preview_state=staged.state.clone();
            preview_card_rows(&preview_state,staged.state.room_loop.as_mut().ok_or(RunError::InvalidAction)?.screen.choices.iter_mut()
                .filter(|choice|choice.status==RoomChoiceStatus::Offered).map(|choice|&mut choice.kind));
            // The Courier restocks before any equip continuation replaces the shop screen.
            if complete_courier_shop(&staged) {
                restock_courier_relic(&mut staged)?;
            }
            let transition=crate::items::lifecycle::after_obtain(&mut staged,id,relic,obtained)?;
            *run=staged;
            return Ok(transition)
        }
    }
    if run.state.item_admission==ItemAdmission::Complete {
        match row.kind.clone(){
            RoomChoiceKind::Potion(potion)=>{
                if run.state.relics.contains(&RelicId::Sozu){return Err(RunError::InvalidAction)}
                if !run.state.potions.iter().any(Option::is_none){return Err(RunError::InvalidAction)}
                let mut staged=run.clone();
                if !crate::items::obtain_run_potion(&mut staged.state,potion)?{return Err(RunError::InvalidAction)}
                staged.state.gold-=price;
                staged.state.room_loop.as_mut().ok_or(RunError::InvalidAction)?.screen.choices.iter_mut()
                    .find(|choice|choice.id==id).ok_or(RunError::InvalidAction)?.status=RoomChoiceStatus::Claimed;
                crate::items::dispatch::run_event(&mut staged.state,&mut crate::items::dispatch::RunItemEvent::ShopPurchase)?;
                if complete_courier_shop(&staged){
                    let row_index=staged.state.room_loop.as_ref().ok_or(RunError::InvalidAction)?.screen.choices.iter()
                        .filter(|choice|matches!(choice.kind,RoomChoiceKind::Potion(_))).count() as u64;
                    restock_courier_potion(&mut staged,row_index)?;
                }
                *run=staged;
                return Ok(empty_transition())
            }
            RoomChoiceKind::Card{id:card,upgrades}=>{
                let mut staged=run.clone();
                mutations::obtain_card(&mut staged.state,card,upgrades)?;
                staged.state.gold-=price;
                staged.state.room_loop.as_mut().ok_or(RunError::InvalidAction)?.screen.choices.iter_mut()
                    .find(|choice|choice.id==id).ok_or(RunError::InvalidAction)?.status=RoomChoiceStatus::Claimed;
                crate::items::dispatch::run_event(&mut staged.state,&mut crate::items::dispatch::RunItemEvent::ShopPurchase)?;
                if complete_courier_shop(&staged){
                    let row_index=staged.state.room_loop.as_ref().ok_or(RunError::InvalidAction)?.screen.choices.iter()
                        .filter(|choice|matches!(choice.kind,RoomChoiceKind::Card{..})).count() as u64;
                    restock_courier_card(&mut staged,card,row_index)?;
                }
                *run=staged;
                return Ok(empty_transition())
            }
            _=>{}
        }
    }
    match row.kind{
        RoomChoiceKind::Card{id,upgrades}=>{mutations::obtain_card(&mut run.state,id,upgrades)?;},
        RoomChoiceKind::Potion(potion)=>{let slot=run.state.potions.iter().position(|p|p.is_none()).ok_or(RunError::InvalidAction)?;run.state.potions[slot]=Some(potion);},
        RoomChoiceKind::Relic(relic)=>{crate::items::obtain_relic(&mut run.state,relic,crate::items::lifecycle::RelicSource::Shop)?;},
        _=>return Err(RunError::InvalidAction),
    }
    run.state.gold-=price;run.state.room_loop.as_mut().unwrap().screen.choices.iter_mut().find(|c|c.id==id).unwrap().status=RoomChoiceStatus::Claimed;
    if run.state.item_admission==ItemAdmission::Complete {
        // M27 seam: shop purchase
        crate::items::dispatch::run_event(&mut run.state,&mut crate::items::dispatch::RunItemEvent::ShopPurchase)?;
    }
    Ok(empty_transition())
}

/// Economy research §3.2/§3.3: buying The Courier or Membership Card runs its discount pass at once over every
/// current priced row (each step rounded), and the removal price is recomputed from its base; Smiling Mask sets 50.
pub(in crate::run) fn reprice_after_discount_relic(run:&mut Run,relic:RelicId)->Result<(),RunError>{
    let factor=match relic {RelicId::TheCourier=>80,RelicId::MembershipCard=>50,RelicId::SmilingMask=>100,_=>return Ok(())};
    let room=run.state.room_loop.as_ref().ok_or(RunError::InvalidAction)?;
    let base=room.purge_cost;
    let mut removal=crate::items::dispatch::RunItemEvent::RemovalPrice{price:base};
    crate::items::dispatch::run_event(&mut run.state,&mut removal)?;
    let crate::items::dispatch::RunItemEvent::RemovalPrice{price:removal}=removal else {unreachable!("removal-price event kept its kind")};
    let room=run.state.room_loop.as_mut().ok_or(RunError::InvalidAction)?;
    for row in room.screen.choices.iter_mut().filter(|row|row.status==RoomChoiceStatus::Offered) {
        match row.kind {
            RoomChoiceKind::Purge=>row.price=Some(removal),
            RoomChoiceKind::Card{..}|RoomChoiceKind::Potion(_)|RoomChoiceKind::Relic(_)=>
                if let Some(price)=row.price{row.price=Some(((u64::from(price)*factor+50)/100) as u32);},
            _=>{}
        }
    }
    Ok(())
}

fn complete_courier_shop(run:&Run)->bool{
    run.state.item_admission==ItemAdmission::Complete&&run.state.relic_pools.is_some()
        &&(run.state.act==4&&run.state.floor==52||is_standard(&run.state))&&run.state.relics.contains(&RelicId::TheCourier)
}

/// The Courier law of the current shop: Act 4 keeps `ACT4_COURIER`; the standard shop reads the
/// live offset and jitters relic and potion bases on `M28ShopPrice` (design §6.3).
fn courier_law(run:&Run)->CourierLaw{
    if !is_standard(&run.state){return ACT4_COURIER}
    CourierLaw{card:crate::run::RunRngScope::M28ShopStock,potion:crate::run::RunRngScope::M28ShopStock,
        offset:run.state.card_rarity_offset.unwrap_or(super::standard::rewards::OFFSET_START),
        item_jitter:Some(crate::run::RunRngScope::M28ShopPrice),colourless:true}
}

pub(in crate::run) fn append_shop_row(run:&mut Run,kind:RoomChoiceKind,price:u32)->Result<(),RunError>{
    let mut replacement=view(&mut run.next_choice,RoomViewKind::Shop,vec![(kind,Some(price))])?;
    let choice=replacement.choices.pop().ok_or(RunError::InvalidAction)?;
    run.state.room_loop.as_mut().ok_or(RunError::InvalidAction)?.screen.choices.push(choice);
    Ok(())
}

/// One Courier restock law (economy research §3.3), shared by the Complete Act 4 shop and the
/// M28 standard shop. Act 4 passes `ACT4_COURIER`, its accepted arguments: the M26 card scope,
/// a fixed offset 5, the fixed tier base prices and no colourless rows.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(in crate::run) struct CourierLaw {
    /// Rarity, identity and jitter draws of a card replacement.
    pub card:crate::run::RunRngScope,
    /// The potion replacement's `rarity_first` draws.
    pub potion:crate::run::RunRngScope,
    /// The card-rarity offset read (never written; design §6.1).
    pub offset:i32,
    /// `Some(scope)`: relic and potion bases carry a fresh `.95–1.05` jitter `G`-rounded on that
    /// scope; `None`: the fixed tier bases.
    pub item_jitter:Option<crate::run::RunRngScope>,
    /// Whether a colourless purchase restocks (rare on strict binary32 `< .3`, else uncommon).
    pub colourless:bool,
}
const ACT4_COURIER:CourierLaw=CourierLaw{card:crate::run::RunRngScope::M26CardOffer,potion:crate::run::RunRngScope::M27PotionOffer,
    offset:5,item_jitter:None,colourless:false};

/// `L(a, b, k) = F(a + F(F(k / 2^24) × F(b − a)))` on the 24-bit grid (economy research :52-59).
pub(in crate::run) fn jitter(low:f32,high:f32,k:u64)->f32{
    let unit=(k as f32)/16_777_216.0_f32;
    low+(high-low)*unit
}
/// The displayed removal price after a paid purge (economy research :180-183): Smiling Mask 50;
/// otherwise `G(F(F(new_base × .8) × .5))` with each owned factor, one rounding. It is display-only:
/// the claimed row grants no second removal in the same shop.
pub(in crate::run) fn next_removal_price(state:&RunState,new_base:u32)->u32{
    if state.relics.contains(&RelicId::SmilingMask){return 50}
    let mut price=new_base as f32;
    if state.relics.contains(&RelicId::TheCourier){price*=0.8_f32;}
    if state.relics.contains(&RelicId::MembershipCard){price*=0.5_f32;}
    libgdx_round(price)
}
/// libGDX `MathUtils.round(float)`, moved unchanged to the shared numeric module (M29a1).
pub(in crate::run) use crate::run::numeric::libgdx_round;
/// A relic or potion sticker `G(F(base × L(F(.95), F(1.05), k)))` (economy research :167-169).
pub(in crate::run) fn item_sticker(base:u32,k:u64)->u32{libgdx_round((base as f32)*jitter(0.95_f32,1.05_f32,k))}
/// Card rarity base price (economy research :167).
pub(in crate::run) fn card_base(rarity:CardRarity)->Result<u32,RunError>{
    match rarity{CardRarity::Common=>Ok(50),CardRarity::Uncommon=>Ok(75),CardRarity::Rare=>Ok(150),_=>Err(RunError::InvalidAction)}
}
fn relic_base(relic:RelicId)->Result<u32,RunError>{
    match crate::content::Catalog::standard().relic_tier(relic).ok_or(RunError::Registry("complete relic tier missing"))? {
        crate::content::RelicTier::Common=>Ok(150),crate::content::RelicTier::Uncommon=>Ok(250),crate::content::RelicTier::Rare=>Ok(300),
        crate::content::RelicTier::Shop=>Ok(150),_=>Err(RunError::InvalidAction),
    }
}
fn shop_price_event(run:&mut Run,base:u32)->Result<u32,RunError>{
    let mut item_event=crate::items::dispatch::RunItemEvent::ShopPrice{price:base};
    crate::items::dispatch::run_event(&mut run.state,&mut item_event)?;
    let crate::items::dispatch::RunItemEvent::ShopPrice{price}=item_event else {unreachable!("shop-price event kept its kind")};
    Ok(price)
}
fn item_price(run:&mut Run,base:u32,key:&[u64],law:CourierLaw)->Result<u32,RunError>{
    let base=match law.item_jitter{
        None=>base,
        Some(scope)=>{
            let jitter_key=[key,&[2]].concat();
            item_sticker(base,run.state.rng.bounded_scoped_checked(scope,&jitter_key,1<<24)? as u64)
        }
    };
    // The Courier and Membership Card passes, each `G`-rounded (M27 `ShopPrice`).
    shop_price_event(run,base)
}

/// Courier potion replacement: one `rarity_first` proposal, then its price (economy research §3.3).
pub(in crate::run) fn courier_potion(run:&mut Run,key:&[u64],law:CourierLaw)->Result<(PotionId,u32),RunError>{
    let character=run.state.character;
    let mut rng=run.state.rng.clone();
    let potion=crate::items::potion_generation::rarity_first(character,|bound|rng.bounded_scoped_checked(law.potion,key,bound))?;
    run.state.rng=rng;
    let base=potion_base_price(potion)?;
    Ok((potion,item_price(run,base,key,law)?))
}

/// Courier card replacement (economy research §3.3): the purchased type, ShopRoom rarity at the
/// read-only offset (a common Power falls through to Uncommon), and base × jitter (× 1.2 if
/// colourless) × .8 × .5 with one final truncation and no sale.
pub(in crate::run) fn courier_card(run:&mut Run,purchased:crate::content::CardId,key:&[u64],law:CourierLaw)->Result<(crate::content::CardId,u32),RunError>{
    let definition=crate::content::Catalog::standard().card(purchased).ok_or(RunError::Registry("complete shop card missing"))?;
    let colourless=definition.color==CardColor::Colorless;
    if colourless&&!law.colourless||!colourless&&!matches!(definition.card_type,CardType::Attack|CardType::Skill|CardType::Power){
        return Err(RunError::InvalidAction)
    }
    let character=run.state.character;
    let (rarity,candidates)=if colourless{
        let roll=run.state.rng.bounded_scoped_checked(law.card,key,1<<24)? as f32/16_777_216.0_f32;
        let rarity=if roll<0.3_f32{CardRarity::Rare}else{CardRarity::Uncommon};
        (rarity,crate::content::pools::cards(crate::content::pools::CardPool::Colourless{rarity:Some(rarity)})
            .map_err(|_|RunError::Unsupported("complete Courier colourless family is incomplete"))?)
    }else{
        let rarity_roll=run.state.rng.bounded_scoped_checked(law.card,key,100)?;
        // The shared card-rarity owner (M28a item 9); a shop is an ordinary room (R 3, U 37).
        let mut rarity=super::standard::rewards::classify(super::standard::rewards::RarityRoom::Ordinary,rarity_roll as u32,law.offset,1);
        let mut candidates=crate::content::pools::cards(crate::content::pools::CardPool::Ordinary{
            character,rarity:Some(rarity),card_type:Some(definition.card_type),
        }).map_err(|_|RunError::Unsupported("complete Courier card family is incomplete"))?;
        if candidates.is_empty()&&rarity==CardRarity::Common&&definition.card_type==CardType::Power{
            // Merchant Power rows have no Common family; source common proposals fall through to Uncommon.
            rarity=CardRarity::Uncommon;
            candidates=crate::content::pools::cards(crate::content::pools::CardPool::Ordinary{
                character,rarity:Some(rarity),card_type:Some(definition.card_type),
            }).map_err(|_|RunError::Unsupported("complete Courier Power family is incomplete"))?;
        }
        (rarity,candidates)
    };
    let identity_key=[key,&[1]].concat();
    let identity=run.state.rng.bounded_scoped_checked(law.card,&identity_key,candidates.len() as u64)?;
    let replacement=candidates[identity];

    // Economy §3.3 keeps the replacement's binary32 discounts together and truncates once.
    let jitter_key=[key,&[2]].concat();
    let k=run.state.rng.bounded_scoped_checked(law.card,&jitter_key,1<<24)? as u64;
    let mut raw=(card_base(rarity)? as f32)*jitter(0.9_f32,1.1_f32,k);
    if colourless{raw*=1.2_f32;}
    Ok((replacement,courier_card_discount(&run.state,raw)))
}

/// A Courier card replacement's discounts on its binary32 price: × .8 Courier, × .5 Membership
/// Card, then one final truncation (economy research §3.3; c-a.md S8: 151 → 60 here, against 61
/// from the sequential shop-entry `G` passes).
pub(in crate::run) fn courier_card_discount(state:&RunState,mut raw:f32)->u32{
    if state.relics.contains(&RelicId::TheCourier){raw*=0.8_f32;}
    if state.relics.contains(&RelicId::MembershipCard){raw*=0.5_f32;}
    raw as u32
}

/// Courier relic replacement: a fresh merchant tier (also in the former Shop slot), the end
/// selector, and Old Coin, Smiling Mask, Maw Bank and The Courier excluded (economy research §3.3).
pub(in crate::run) fn courier_relic(run:&mut Run,key:&[u64],law:CourierLaw)->Result<(RelicId,u32),RunError>{
    let tier=crate::items::relic_pools::roll_tier(&mut run.state,crate::items::relic_pools::TierTable::Merchant,key);
    let relic=crate::items::relic_pools::exclude_take(&mut run.state,tier,crate::items::relic_pools::Selector::Back,
        crate::items::lifecycle::RelicSource::Shop,
        &[RelicId::OldCoin,RelicId::SmilingMask,RelicId::MawBank,RelicId::TheCourier],key)?;
    let base=relic_base(relic)?;
    Ok((relic,item_price(run,base,key,law)?))
}

fn shop_node_key(run:&Run,row_index:u64,source_tag:u64)->Result<[u64;5],RunError>{
    let node=run.state.map.as_ref().and_then(|map|map.current).ok_or(RunError::InvalidAction)?;
    Ok([u64::from(run.state.act),u64::from(run.state.floor),node.0,row_index,source_tag])
}

fn restock_courier_potion(run:&mut Run,row_index:u64)->Result<(),RunError>{
    let key=shop_node_key(run,row_index,COMPLETE_COURIER_POTION_SOURCE_TAG)?;
    let (potion,price)=courier_potion(run,&key,courier_law(run))?;
    append_shop_row(run,RoomChoiceKind::Potion(potion),price)
}

fn restock_courier_card(run:&mut Run,purchased:crate::content::CardId,row_index:u64)->Result<(),RunError>{
    let key=shop_node_key(run,row_index,COMPLETE_COURIER_CARD_SOURCE_TAG)?;
    let (card,price)=courier_card(run,purchased,&key,courier_law(run))?;
    let upgrades=super::standard::rewards::preview_upgrades(&run.state,card,0);
    append_shop_row(run,RoomChoiceKind::Card{id:card,upgrades},price)
}

fn restock_courier_relic(run:&mut Run)->Result<(),RunError>{
    let key=shop_node_key(run,0,COMPLETE_COURIER_RELIC_SOURCE_TAG)?;
    let (relic,price)=courier_relic(run,&key,courier_law(run))?;
    append_shop_row(run,RoomChoiceKind::Relic(relic),price)
}
