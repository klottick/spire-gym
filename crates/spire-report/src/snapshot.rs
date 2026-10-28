use crate::ReportError;
use serde_json::{json,Value};
use spire_core::combat::*;
pub fn run_snapshot(d:&spire_core::run::PublicDecision,map:Option<&spire_core::run::map::PublicMap>)->Result<Value,ReportError> {
    let id=combat_id(d);
    let mut player=json!({"hp":d.player.hp,"maxHp":d.player.max_hp,"gold":d.player.gold,"block":null,"energy":null,"statuses":null});
    let mut inventory=inventory(&d.player.relics,&d.player.relic_counters,&d.player.potions);
    if let Some(c)=&d.combat {
        // The combat projection is the sole live owner; never fall back to a
        // persistent checkpoint's stale resources or item counters.
        player=full_player(c);player["gold"]=json!(d.player.gold);
        inventory=c.inventory.as_ref().map(|i|inventory_value(i)).unwrap_or(Value::Null);
    }
    if inventory.is_object() {
        inventory["deck"]=json!(d.objects.iter().map(permanent_card).collect::<Vec<_>>());
        inventory["keys"]=json!(d.player.keys);
    }
    Ok(json!({"screen":crate::actions::screen(d,&id)?,"player":player,"inventory":inventory,
        "combat":d.combat.as_ref().map(|c|native_combat(c,&id)).transpose()?,"terminal":d.terminal.map(terminal),
        "map":map.map(|m|json!({"act":m.act,"boss":m.boss.map(|b|b.as_str()),"current":m.current.map(|id|id.0.to_string()),
            "wingedMoves":m.winged_moves,"visited":m.nodes.iter().filter(|n|n.visited).map(|n|n.id.0.to_string()).collect::<Vec<_>>(),
            "nodes":m.nodes.iter().map(|n|json!({"id":n.id.0.to_string(),"floor":n.floor,"x":n.x,"y":n.y,"kind":n.kind.label(),"visited":n.visited,"revealed":n.revealed,
                "resolvedRoom":if n.visited&&n.revealed {n.resolved_room.map(|k|k.label())}else{None},"burning":n.burning})).collect::<Vec<_>>(),
            "edges":m.nodes.iter().flat_map(|n|n.edges.iter().map(move |id|json!({"from":n.id.0.to_string(),"to":id.0.to_string()}))).collect::<Vec<_>>()}))}))
}
pub(crate) fn combat_id(d:&spire_core::run::PublicDecision)->String {format!("combat-{}",d.combat_progress.entered.saturating_sub(1))}
pub(crate) fn terminal(t:spire_core::run::RunTerminal)->&'static str {use spire_core::run::RunTerminal::*;match t {HeartVictory=>"heart_victory",Death=>"death",EmptyDeck=>"empty_deck",Act3NoKeys=>"act3_no_keys"}}
pub(crate) fn permanent_card(c:&spire_core::run::PublicObject)->Value {
    json!({"ref":format!("permanent/card/{}",c.instance),"origin":{"kind":"permanent","instance":c.instance.to_string()},"cardId":c.id.as_str(),"name":c.id.as_str(),"upgrades":c.upgrades,
        "baseCost":null,"currentCost":null,"freeToPlay":null,"assetKey":format!("card/{}",c.id.as_str())})
}
pub(crate) fn live_card(c:&LiveCardView,combat:&str)->Value {
    let origin=match c.origin {CombatCardRef::Permanent(id)=>json!({"kind":"permanent","instance":id.0.to_string()}),CombatCardRef::CombatLocal(id)=>json!({"kind":"combat_local","instance":id.to_string()})};
    let values=c.values.as_ref();
    json!({"ref":format!("{combat}/card/{}",c.instance.0),"origin":origin,"cardId":c.id.as_str(),"name":c.id.as_str(),"upgrades":c.upgrades,
        "baseCost":null,"currentCost":c.effective_cost,"freeToPlay":values.map(|v|v.free_play),
        "baseDamage":values.and_then(|v|v.base_damage),"baseBlock":values.and_then(|v|v.base_block),
        "magicNumber":values.and_then(|v|v.magic_number),"damageBonus":values.and_then(|v|v.damage_bonus),
        "costKind":values.map(|v|match v.cost_kind {CardCostKind::Fixed=>"fixed",CardCostKind::X=>"x",CardCostKind::Unplayable=>"unplayable"}),
        "exhaustOnFinish":values.map(|v|v.exhaust_on_finish),"ethereal":values.map(|v|v.ethereal),"innate":values.map(|v|v.innate),
        "temporaryCost":values.map(|v|v.temporary_cost),"assetKey":format!("card/{}",c.id.as_str())})
}
fn full_player(c:&CombatPublic)->Value {json!({"hp":c.player.hp,"maxHp":c.player.max_hp,"block":c.player.block,"energy":c.player.energy,"gold":null,
    "statuses":c.player.powers.iter().map(|p|json!({"id":p.id.0,"amount":p.amount})).collect::<Vec<_>>()})}
fn inventory(relics:&[spire_core::content::RelicId],counters:&[spire_core::items::RelicCounterView],potions:&[Option<spire_core::content::PotionId>])->Value {
    json!({"deck":null,"keys":null,"relics":relics.iter().map(|id|json!({"relicId":id.as_str(),"name":id.as_str(),"assetKey":format!("relic/{}",id.as_str()),
        "counters":counters.iter().filter(|c|c.relic==*id).map(|c|("counter".to_owned(),json!(c.value))).collect::<serde_json::Map<String,Value>>()})).collect::<Vec<_>>(),
        "potions":potions.iter().map(|p|p.map(|id|json!({"potionId":id.as_str(),"name":id.as_str(),"assetKey":format!("potion/{}",id.as_str())}))).collect::<Vec<_>>()})
}
fn inventory_value(i:&CombatInventoryView)->Value {inventory(&i.relics,&i.relic_counters,&i.potions)}
fn draw_order_known(c:&CombatPublic)->bool {
    let count=c.draw_count;
    if count==0{return false;}
    let mut positions=std::collections::BTreeSet::new();
    for known in &c.known_draw{
        if known.offset>=count{return false;}
        positions.insert(if known.from_top{known.offset}else{count-1-known.offset});
    }
    positions.len()==count
}
pub(crate) fn native_combat(c:&CombatPublic,id:&str)->Result<Value,ReportError> {
    if c.targets.len()!=c.enemies.len(){return Err(ReportError::Invalid("public target/actor lengths disagree".into()));}
    Ok(json!({"id":id,"turn":c.turn,"player":full_player(c),"inventory":c.inventory.as_ref().map(inventory_value),"enemies":c.enemies.iter().zip(&c.targets).map(|(e,(target,eligible))|json!({
        "ref":crate::actions::actor_ref(&Actor::Enemy(e.instance_id.clone()),id),"targetId":target.0.to_string(),"targetEligible":eligible,"logicalSlot":e.logical_slot,"monsterId":e.enemy_id.as_str(),"name":e.enemy_id.as_str(),
        "hp":e.hp,"maxHp":e.max_hp,"block":e.block,"statuses":e.powers.iter().map(|p|json!({"id":p.id.0,"amount":p.amount})).collect::<Vec<_>>(),
        "lifeState":if e.replaced {"replaced"}else if e.escaped {"escaped"}else if e.half_dead {"half_dead"}else if e.hp<=0 {"defeated"}else{"alive"},
        "side":c.encounter_facing.as_ref().and_then(|facing|facing.enemies.iter().find(|actor|actor.instance_id==e.instance_id)).map(|actor|match actor.side {EncounterSide::Left=>"left",EncounterSide::Right=>"right"}),
        "intent":if !e.replaced&&!e.escaped&&e.hp>0 {json!({"name":e.intent.as_str(),"damagePerHit":e.intent_damage,"hitCount":intent_hits(e.intent.as_str())})}else{Value::Null},"assetKey":format!("enemy/{}",e.enemy_id.as_str())})).collect::<Vec<_>>(),
        "hand":c.hand.iter().map(|c|live_card(c,id)).collect::<Vec<_>>(),
        "piles":{"draw":{"count":c.draw_count,"knownCards":if c.known_draw.is_empty(){Value::Null}else{json!(c.known_draw.iter().map(|known|json!({"card":live_card(&known.card,id),"fromTop":known.from_top,"offset":known.offset})).collect::<Vec<_>>())},"orderKnown":draw_order_known(c)},"discard":c.discard.iter().map(|c|live_card(c,id)).collect::<Vec<_>>(),"exhaust":c.exhaust.iter().map(|c|live_card(c,id)).collect::<Vec<_>>()},
        "selection":c.selection.as_ref().map(|s|json!({"kind":disposition(s.disposition),"purpose":crate::actions::selection_purpose(s.operation),"required":s.required,"minimum":s.minimum,"maximum":s.maximum,"selected":s.selected.iter().map(|c|c.0.to_string()).collect::<Vec<_>>(),"cards":s.candidates.iter().map(|c|live_card(c,id)).collect::<Vec<_>>()})),
        "character":{"stance":match c.character.stance {Stance::Neutral=>"neutral",Stance::Wrath=>"wrath",Stance::Calm=>"calm"},"focus":c.character.focus,
            "orbs":c.character.orbs.iter().map(|o|o.map(|o|match o {OrbKind::Lightning=>"lightning",OrbKind::Frost=>"frost",OrbKind::Dark=>"dark",OrbKind::Plasma=>"plasma"})).collect::<Vec<_>>()},
        "stasis":c.stasis.iter().map(|s|json!({"orb":crate::actions::actor_ref(&Actor::Enemy(s.orb.clone()),id),"card":live_card(&s.card,id)})).collect::<Vec<_>>(),"lastPlayedCard":null,
        "facing":c.encounter_facing.as_ref().map(|facing|json!({"player":match facing.player_facing {PlayerFacing::Left=>"left",PlayerFacing::Right=>"right"},"surrounded":facing.surrounded,
            "enemies":facing.enemies.iter().map(|actor|json!({"side":match actor.side {EncounterSide::Left=>"left",EncounterSide::Right=>"right"},"backAttack":actor.back_attack})).collect::<Vec<_>>()})),
        "hpBeforeHealing":c.hp_before_healing,"hpAfterCleanup":c.hp_after_cleanup,"outcome":c.outcome.map(|o|match o {Outcome::Victory=>"victory",Outcome::Death=>"death",Outcome::Escaped=>"escaped"})}))
}
fn intent_hits(name:&str)->Option<u8>{match name {"shield_bash"|"shield_smash"|"heart_echo"=>Some(1),"spear_burn_strike"=>Some(2),"spear_skewer"=>Some(3),"heart_blood_shots"=>Some(12),_=>None}}
fn disposition(d:SelectionDisposition)->&'static str {match d {SelectionDisposition::Hand=>"hand",SelectionDisposition::Discard=>"discard",SelectionDisposition::Exhaust=>"exhaust",SelectionDisposition::DrawTop=>"draw_top"}}

pub fn legacy_snapshot(public: &PublicView)->Result<Value,ReportError> {legacy_snapshot_for(public,"combat-0")}
pub(crate) fn legacy_card(card:&CardView,combat:&str)->Value {
    let stats=card_stats(card.card_id,card.upgraded);
    json!({"ref":crate::actions::card_ref(&card.instance_id,combat),"origin":null,"cardId":card.card_id.as_str(),
        "name":card.card_id.as_str(),"upgrades":u32::from(card.upgraded),"baseCost":stats.cost,"currentCost":stats.cost,"freeToPlay":null,"assetKey":format!("card/{}",card.card_id.as_str())})
}
pub(crate) fn legacy_snapshot_for(p:&PublicView,combat:&str)->Result<Value,ReportError> {
    let cards=|cs:&[CardView]|cs.iter().map(|c|legacy_card(c,combat)).collect::<Vec<_>>();
    Ok(json!({"screen":{"kind":"combat","name":"Combat","options":[],"selection":null},
        "player":{"hp":p.player.hp,"maxHp":p.player.max_hp,"block":p.player.block,"energy":p.player.energy,"gold":null,
            "statuses":p.player.powers.iter().map(|(id,n)|json!({"id":id.as_str(),"amount":n})).collect::<Vec<_>>()},
        "inventory":null,"map":null,"terminal":null,
        "combat":{"id":combat,"turn":p.turn,"enemies":p.enemies.iter().map(|e|json!({
            "ref":crate::actions::actor_ref(&Actor::Enemy(e.instance_id.clone()),combat),"logicalSlot":null,"monsterId":e.enemy_id.as_str(),"name":e.enemy_id.as_str(),
            "hp":e.hp,"maxHp":e.max_hp,"block":e.block,"statuses":e.powers.iter().map(|(id,n)|json!({"id":id.as_str(),"amount":n})).collect::<Vec<_>>(),
            "lifeState":if e.hp>0 {"alive"}else{"defeated"},"intent":if e.hp>0 {json!({"name":e.intent_id.as_str(),"damagePerHit":e.intent_damage,"hitCount":null})}else{Value::Null},"assetKey":format!("enemy/{}",e.enemy_id.as_str())})).collect::<Vec<_>>(),
            "hand":cards(&p.hand),"piles":{"draw":{"count":p.draw_count,"knownCards":null,"orderKnown":false},"discard":cards(&p.discard),"exhaust":cards(&p.exhaust)},
            "selection":null,"character":null,"stasis":null,"lastPlayedCard":null,
            "hpBeforeHealing":p.hp_before_healing,"hpAfterCleanup":p.hp_after_cleanup,
            "outcome":p.outcome.map(|o|match o {Outcome::Victory=>"victory",Outcome::Death=>"death",Outcome::Escaped=>"escaped"})}}))
}
