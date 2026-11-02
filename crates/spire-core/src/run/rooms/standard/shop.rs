//! Standard shop (design §6.3), reached only from `RoomScenario::StandardA0`.
//! Lane body: see `m28a/task0/spec-sheets/c-a.md` rules S1.. .
use super::*;
use crate::content::{CardId, CardRarity, CardType, PotionId, PotionRarity, RelicId, RelicTier};

const SHOP_STOCK_SOURCE_TAG: u64 = 0;
const SHOP_PRICE_SOURCE_TAG: u64 = 1;

/// Enter the shop at the current map node: build the standard stock (`M28ShopStock`,
/// `M28ShopPrice`, slot order cards, colourless, relics, potions; design §6.3) and show it.
pub(in crate::run) fn enter(run:&mut Run)->Result<RunTransition,RunError>{
    if !is_standard(&run.state) || run.state.item_admission != ItemAdmission::Complete {
        return Err(RunError::InvalidAction);
    }
    let mut staged = run.clone();
    let map = staged.state.map.as_ref().ok_or(RunError::InvalidAction)?;
    let node_id = map.current.ok_or(RunError::InvalidAction)?;
    let node = map.nodes.iter().find(|node| node.id == node_id).ok_or(RunError::InvalidAction)?;
    // A question room that resolved to a shop records the shop as its hidden kind (design §6.2).
    if node.hidden_kind.unwrap_or(node.kind) != MapNodeKind::Shop || node.floor != staged.state.floor || map.act != staged.state.act {
        return Err(RunError::InvalidAction);
    }
    let destination = (staged.state.act, staged.state.floor, node_id);
    let offset = staged.state.card_rarity_offset.ok_or(RunError::InvalidAction)?;
    let mut rows = Vec::with_capacity(15);

    // Five typed colored offers. The second proposal of each type retries both laws on an
    // attempt-specific key when it matches the first offer of that type (economy research :155-157).
    let card_types = [CardType::Attack, CardType::Attack, CardType::Skill, CardType::Skill, CardType::Power];
    let mut first_attack = None;
    let mut first_skill = None;
    for (slot, card_type) in card_types.into_iter().enumerate() {
        let forbidden = match card_type {
            CardType::Attack if slot == 1 => first_attack,
            CardType::Skill if slot == 3 => first_skill,
            _ => None,
        };
        let (card, rarity) = colored_card(&mut staged, destination, slot as u64, offset, card_type, forbidden)?;
        if slot == 0 { first_attack = Some(card); }
        if slot == 2 { first_skill = Some(card); }
        let price = card_sticker(&mut staged, destination, slot as u64, rarity, false)?;
        rows.push((RoomChoiceKind::Card { id: card, upgrades: 0 }, Some(price)));
    }

    // The colorless rows are fixed Uncommon then Rare, with equal identity mass and no ownership exclusion.
    for (slot, rarity) in [(5_u64, CardRarity::Uncommon), (6, CardRarity::Rare)] {
        let key = shop_key(destination, SHOP_STOCK_SOURCE_TAG, slot, 0);
        let candidates = crate::content::pools::cards(crate::content::pools::CardPool::Colourless { rarity: Some(rarity) })
            .map_err(|_| RunError::Unsupported("complete shop colorless family is incomplete"))?;
        if candidates.is_empty() { return Err(RunError::Unsupported("complete shop colorless family is empty")); }
        let index = staged.state.rng.bounded_scoped_checked(RunRngScope::M28ShopStock, &key, candidates.len() as u64)?;
        let card = candidates[index];
        let price = card_sticker(&mut staged, destination, slot, rarity, true)?;
        rows.push((RoomChoiceKind::Card { id: card, upgrades: 0 }, Some(price)));
    }

    // Preserve live relic-pool consumption. R1 applies Back to all three initial relic slots.
    for slot in 7_u64..=9 {
        let key = shop_key(destination, SHOP_STOCK_SOURCE_TAG, slot, 0);
        let relic = if slot < 9 {
            let tier = crate::items::relic_pools::roll_tier(&mut staged.state,
                crate::items::relic_pools::TierTable::Merchant, &key);
            crate::items::relic_pools::take(&mut staged.state, tier,
                crate::items::relic_pools::Selector::Back, crate::items::lifecycle::RelicSource::Shop, &key)?
        } else {
            crate::items::relic_pools::take(&mut staged.state, RelicTier::Shop,
                crate::items::relic_pools::Selector::Back, crate::items::lifecycle::RelicSource::Shop, &key)?
        };
        let base = relic_base(relic)?;
        let price = item_sticker(&mut staged, destination, slot, base)?;
        rows.push((RoomChoiceKind::Relic(relic), Some(price)));
    }

    // Three unrestricted rarity-first proposals; duplicates and offers blocked by Sozu/full belt remain.
    let character = staged.state.character;
    for slot in 10_u64..=12 {
        let key = shop_key(destination, SHOP_STOCK_SOURCE_TAG, slot, 0);
        let potion = crate::items::potion_generation::rarity_first(character, |bound| {
            staged.state.rng.bounded_scoped_checked(RunRngScope::M28ShopStock, &key, bound)
        })?;
        let base = potion_base(potion)?;
        let price = item_sticker(&mut staged, destination, slot, base)?;
        rows.push((RoomChoiceKind::Potion(potion), Some(price)));
    }

    // One uniformly chosen colored offer is half price before item-discount passes.
    let sale_key = shop_key(destination, SHOP_PRICE_SOURCE_TAG, 13, 0);
    let sale_slot = staged.state.rng.bounded_scoped_checked(RunRngScope::M28ShopPrice, &sale_key, 5)?;
    rows[sale_slot].1 = Some(rows[sale_slot].1.ok_or(RunError::InvalidAction)? / 2);
    rows.push((RoomChoiceKind::Purge, Some(staged.state.room_loop.as_ref().ok_or(RunError::InvalidAction)?.purge_cost)));
    rows.push((RoomChoiceKind::Leave, None));

    staged.state.room = "Shop";
    let screen = crate::run::rooms::shop::standard_screen(&mut staged, rows)?;
    staged.state.room_loop.as_mut().ok_or(RunError::InvalidAction)?.screen = screen;
    *run = staged;
    Ok(crate::run::rooms::empty_transition())
}

fn shop_key(destination:(u8,u32,NodeId),source_tag:u64,slot:u64,attempt:u64)->[u64;7]{
    [u64::from(destination.0),u64::from(destination.1),destination.2.0,0,source_tag,slot,attempt]
}

fn colored_card(run:&mut Run,destination:(u8,u32,NodeId),slot:u64,offset:i32,card_type:CardType,
    forbidden:Option<CardId>)->Result<(CardId,CardRarity),RunError>{
    let mut attempt=0_u64;
    loop {
        let key=shop_key(destination,SHOP_STOCK_SOURCE_TAG,slot,attempt);
        let roll=run.state.rng.bounded_scoped_checked(RunRngScope::M28ShopStock,&key,100)? as u32;
        let mut rarity=super::rewards::classify(super::rewards::RarityRoom::Ordinary,roll,offset,1);
        let mut candidates=crate::content::pools::cards(crate::content::pools::CardPool::Ordinary{
            character:run.state.character,rarity:Some(rarity),card_type:Some(card_type),
        }).map_err(|_|RunError::Unsupported("complete shop typed card family is incomplete"))?;
        if candidates.is_empty()&&rarity==CardRarity::Common&&card_type==CardType::Power {
            rarity=CardRarity::Uncommon;
            candidates=crate::content::pools::cards(crate::content::pools::CardPool::Ordinary{
                character:run.state.character,rarity:Some(rarity),card_type:Some(card_type),
            }).map_err(|_|RunError::Unsupported("complete shop Power family is incomplete"))?;
        }
        if candidates.is_empty(){return Err(RunError::Unsupported("complete shop typed card family is empty"));}
        let identity=run.state.rng.bounded_scoped_checked(RunRngScope::M28ShopStock,&key,candidates.len() as u64)?;
        let card=candidates[identity];
        if Some(card)==forbidden {
            attempt=attempt.checked_add(1).ok_or(RunError::Unsupported("shop proposal attempt exhausted"))?;
            continue;
        }
        return Ok((card,rarity));
    }
}

fn card_sticker(run:&mut Run,destination:(u8,u32,NodeId),slot:u64,rarity:CardRarity,colorless:bool)->Result<u32,RunError>{
    let key=shop_key(destination,SHOP_PRICE_SOURCE_TAG,slot,0);
    let k=run.state.rng.bounded_scoped_checked(RunRngScope::M28ShopPrice,&key,1<<24)? as u64;
    let mut raw=(crate::run::rooms::shop::card_base(rarity)? as f32)
        *crate::run::rooms::shop::jitter(0.9_f32,1.1_f32,k);
    if colorless { raw*=1.2_f32; }
    Ok(raw as u32)
}

fn item_sticker(run:&mut Run,destination:(u8,u32,NodeId),slot:u64,base:u32)->Result<u32,RunError>{
    let key=shop_key(destination,SHOP_PRICE_SOURCE_TAG,slot,0);
    let k=run.state.rng.bounded_scoped_checked(RunRngScope::M28ShopPrice,&key,1<<24)? as u64;
    Ok(crate::run::rooms::shop::item_sticker(base,k))
}

fn relic_base(id:RelicId)->Result<u32,RunError>{
    match crate::content::Catalog::standard().relic_tier(id).ok_or(RunError::Registry("complete relic tier missing"))? {
        RelicTier::Common|RelicTier::Shop=>Ok(150),
        RelicTier::Uncommon=>Ok(250),
        RelicTier::Rare=>Ok(300),
        _=>Err(RunError::InvalidAction),
    }
}

fn potion_base(id:PotionId)->Result<u32,RunError>{
    match crate::content::Catalog::standard().potion_rarity(id).ok_or(RunError::Registry("complete potion rarity missing"))? {
        PotionRarity::Common=>Ok(50),
        PotionRarity::Uncommon=>Ok(75),
        PotionRarity::Rare=>Ok(100),
    }
}

// Purchases, the purge and Leave go through the shared `rooms::shop::choose` (economy research
// §3.3), whose Courier restock reads `courier_law` (standard: live offset, M28 scopes).

/// Diagnostics and crate tests: a standard A0 Run that took the short Neow max-HP blessing and
/// stands on an Act 1 shop node with the supplied gold, deck, relics and potions, then enters
/// the shop. The only way to reach a standard shop until M28b's map rooms.
#[cfg(any(test,feature="m28-diagnostics"))]
pub(in crate::run) fn diagnostic_shop(character:crate::content::Character,supply:ShopSupply,rng:RunRng)->Result<Run,RunError>{
    let profile=CreationProfile{neow_eligible:false,..CreationProfile::experienced()};
    let mut run=Run::new_room_scenario(RunConfig{character,ascension:0},profile,rng,RoomScenario::StandardA0)?;
    let max_hp=run.state.room_loop.as_ref().ok_or(RunError::InvalidAction)?.screen.choices.iter()
        .find(|row|matches!(row.kind,RoomChoiceKind::NeowMaxHp{..})).ok_or(RunError::InvalidAction)?.id;
    run.step(&GameAction::Choose(max_hp))?;
    let map=run.state.map.as_mut().ok_or(RunError::InvalidAction)?;
    let node=map.nodes.iter().find(|node|node.kind==MapNodeKind::Shop).ok_or(RunError::Unsupported("Act 1 map has no shop"))?.clone();
    map.current=Some(node.id);
    run.state.floor=node.floor;
    run.state.room="Shop";
    run.state.gold=supply.gold;
    if let Some(deck)=supply.deck{
        run.state.deck.clear();
        for (id,upgrades) in deck{
            let instance=run.state.next_instance;run.state.next_instance+=1;
            run.state.deck.push(CardInstance{misc:0,id,instance,upgrades});
        }
    }
    for relic in supply.relics{
        if let Some(pools)=run.state.relic_pools.as_mut(){crate::items::relic_pools::remove(pools,relic);}
        run.state.relics.push(relic);
    }
    if let Some(potions)=supply.potions{run.state.potions=potions;}
    run.state.room_loop.as_mut().ok_or(RunError::InvalidAction)?.purge_cost=supply.purge_cost;
    enter(&mut run)?;
    Ok(run)
}

/// The supplied shop-entry state of `diagnostic_shop`.
#[cfg(any(test,feature="m28-diagnostics"))]
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct ShopSupply {
    pub gold:u32,
    /// `None` keeps the starter deck.
    pub deck:Option<Vec<(crate::content::CardId,u32)>>,
    /// Relics appended after the starter relic (no obtain hooks), removed from the pools.
    pub relics:Vec<crate::content::RelicId>,
    pub potions:Option<Vec<Option<crate::content::PotionId>>>,
    /// The run-wide removal base (75 at Run start).
    pub purge_cost:u32,
}
