//! Typed native identities and resolved values only; no browser-side game rules.
use serde_json::{json,Value};
use spire_core::combat::*;
use spire_core::run::*;
use crate::ReportError;
pub fn describe_run_action(d:&PublicDecision,a:&GameAction,id:&str)->Result<Value,ReportError> {
    let candidate=d.candidates.iter().find(|c|&c.action==a).ok_or_else(||ReportError::Invalid("action not offered in public decision".into()))?;
    let mut value=json!({"type":action_type(a),"description":candidate.label,"source":"player","targets":[],"card":null,"optionKey":option_key(a),"preActionOffer":null,"parentActionId":null});
    match a {
        GameAction::Choose(choice)=> {
            if let Some(row)=d.room.as_ref().and_then(|r|r.choices.iter().find(|r|r.id==*choice)) {
                let (label,data)=room_data(row);value["description"]=json!(label);value["preActionOffer"]=data;
            }else if let Some(card)=&candidate.object {value["preActionOffer"]=crate::snapshot::permanent_card(card);}
        },
        GameAction::Move(node)=>{value["description"]=json!(format!("Move to {}",node.0));value["preActionOffer"]=json!({"nodeId":node.0.to_string()});},
        GameAction::Select(_)=>if let Some(card)=&candidate.object {value["preActionOffer"]=crate::snapshot::permanent_card(card);},
        GameAction::UsePotion {slot,..}|GameAction::DiscardPotion {slot}=> {value["preActionOffer"]=json!({"slot":slot,"potionId":d.player.potions.get(*slot as usize).and_then(|p|*p).map(|p|p.as_str())});},
        GameAction::Combat(action)=> {
            let c=d.combat.as_ref().ok_or_else(||ReportError::Invalid("combat action without public combat".into()))?;
            if c.targets.len()!=c.enemies.len(){return Err(ReportError::Invalid("public target/actor lengths disagree".into()));}
            let target=match action {CombatAction::PlayCard{target,..}|CombatAction::UsePotion{target,..}=>*target,_=>None};
            if let Some(t)=target {
                let e=c.targets.iter().zip(&c.enemies).find(|((id,eligible),_)|*id==t&&*eligible).map(|(_,e)|e)
                    .ok_or_else(||ReportError::Invalid("public target not eligible".into()))?;
                value["targets"]=json!([actor_ref(&Actor::Enemy(e.instance_id.clone()),id)]);
            }
            match action {
                CombatAction::PlayCard {instance,..}=> {
                    let card=c.hand.iter().find(|c|c.instance.0==*instance).ok_or_else(||ReportError::Invalid("played card not in public hand".into()))?;
                    value["card"]=crate::snapshot::live_card(card,id);value["source"]=value["card"]["ref"].clone();value["description"]=json!(format!("Play {}",card.id.as_str()));
                },
                CombatAction::SelectCard {instance}=> {
                    let card=c.selection.as_ref().and_then(|s|s.candidates.iter().find(|c|c.instance.0==*instance)).ok_or_else(||ReportError::Invalid("selected card not public".into()))?;
                    value["preActionOffer"]=crate::snapshot::live_card(card,id);value["description"]=json!(format!("Select {}",card.id.as_str()));
                },
                CombatAction::UsePotion {slot,..}=>{value["preActionOffer"]=json!({"slot":slot,"potionId":c.inventory.as_ref().and_then(|i|i.potions.get(*slot)).and_then(|p|*p).map(|p|p.as_str())});value["description"]=json!("Use potion");},
                CombatAction::EndTurn=>value["description"]=json!("End Turn"),
                CombatAction::ConfirmSelection=>value["description"]=json!("Confirm selection"),
            }
        },_=>(),
    }
    Ok(value)
}
fn action_type(a:&GameAction)->&'static str {match a {GameAction::Move(_)=>"move",GameAction::Choose(_)=>"choose",GameAction::Select(_)=>"select",GameAction::Confirm=>"confirm",GameAction::Cancel=>"cancel",GameAction::UsePotion{..}=>"use_potion",GameAction::DiscardPotion{..}=>"discard_potion",GameAction::Combat(c)=>match c {CombatAction::PlayCard{..}=>"play_card",CombatAction::SelectCard{..}=>"select_card",CombatAction::UsePotion{..}=>"combat_potion",CombatAction::EndTurn=>"end_turn",CombatAction::ConfirmSelection=>"confirm_selection"}}}
pub(crate) fn option_key(a:&GameAction)->String {
    let target=|t:Option<TargetId>|t.map(|t|t.0.to_string()).unwrap_or_else(||"none".into());
    match a {
        GameAction::Move(id)=>format!("move/{}",id.0),GameAction::Choose(id)=>format!("choose/{}",id.0),GameAction::Select(id)=>format!("select/{}",id.0),
        GameAction::Confirm=>"confirm".into(),GameAction::Cancel=>"cancel".into(),
        GameAction::UsePotion{slot,target:t}=>format!("use_potion/{slot}/{}",target(*t)),GameAction::DiscardPotion{slot}=>format!("discard_potion/{slot}"),
        GameAction::Combat(c)=>match c {CombatAction::EndTurn=>"combat/end_turn".into(),CombatAction::PlayCard{instance,target:t}=>format!("combat/play/{instance}/{}",target(*t)),
            CombatAction::UsePotion{slot,target:t}=>format!("combat/potion/{slot}/{}",target(*t)),CombatAction::SelectCard{instance}=>format!("combat/select/{instance}"),CombatAction::ConfirmSelection=>"combat/confirm_selection".into()},
    }
}
fn room_data(row:&RoomChoiceView)->(String,Value) {
    use RoomChoiceKind::*;
    let (label,mut data)=match &row.kind {
        Card{id,upgrades}=>(id.as_str().to_owned(),json!({"cardId":id.as_str(),"upgrades":upgrades})),Potion(id)=>(id.as_str().into(),json!({"potionId":id.as_str()})),Relic(id)=>(id.as_str().into(),json!({"relicId":id.as_str()})),
        NeowMaxHp{amount}=>("Neow max HP".into(),json!({"amount":amount})),Gold(amount)=>("Gold".into(),json!({"amount":amount})),Rest{heal}=>("Rest".into(),json!({"heal":heal})),
        JoustResult{owner_wins}=>("Joust result".into(),json!({"ownerWins":owner_wins})),
        FallingLose{card_type,instance}=>("Lose card".into(),json!({"cardType":match card_type {spire_core::content::CardType::Attack=>"attack",spire_core::content::CardType::Skill=>"skill",spire_core::content::CardType::Power=>"power",spire_core::content::CardType::Status=>"status",spire_core::content::CardType::Curse=>"curse"},"instance":instance.map(|id|id.to_string())})),
        TombGold=>("Take gold".into(),json!({})),TombOfferGold=>("Offer gold".into(),json!({})),
        Act3Continue{stage}=>("Continue".into(),json!({"stage":stage})),
        kind=>(match kind {NeowLament=>"Neow's Lament",CardReward=>"Card reward",Sapphire=>"Sapphire key",Emerald=>"Emerald key",OpenChest=>"Open chest",Smith=>"Smith",Recall=>"Recall",Purge=>"Remove card",ShrineUpgrade=>"Upgrade card",ShrineRemove=>"Remove card",AncientElegance=>"Elegance",AncientSimplicity=>"Simplicity",JoustMurderer=>"Bet on murderer",JoustOwner=>"Bet on owner",JoustContinue=>"Continue Joust",Leave=>"Leave",Proceed=>"Proceed",Skip=>"Skip",ConfirmSkip=>"Confirm skip",CancelSkip=>"Cancel skip",_=>unreachable!()}.into(),json!({})),
    };
    data["price"]=json!(row.price);data["choiceId"]=json!(row.id.0.to_string());(label,data)
}
pub(crate) fn screen(d:&PublicDecision,id:&str)->Result<Value,ReportError> {
    let (kind,name)=d.room.as_ref().map(|r|room_name(r.kind)).unwrap_or_else(||if d.combat.is_some(){("combat","Combat".into())}else{("decision","Decision".into())});
    let mut options=Vec::new();
    if let Some(room)=&d.room {for row in &room.choices {
        let (label,data)=room_data(row);options.push(json!({"key":option_key(&GameAction::Choose(row.id)),"label":label,"status":match row.status {RoomChoiceStatus::Offered=>"offered",RoomChoiceStatus::Claimed=>"claimed",RoomChoiceStatus::Skipped=>"skipped",RoomChoiceStatus::Disabled=>"disabled"},"data":data}));
    }}
    for candidate in &d.candidates {
        let key=option_key(&candidate.action);
        if options.iter().any(|o|o["key"]==key){continue;}
        let a=describe_run_action(d,&candidate.action,id)?;
        options.push(json!({"key":key,"label":a["description"],"status":"offered","data":a["preActionOffer"]}));
    }
    Ok(json!({"kind":kind,"name":name,"options":options,"selection":d.selection.as_ref().map(|s|json!({"kind":"permanent_cards","required":s.required,"selectedCount":s.selected_count,"cards":s.eligible.iter().map(crate::snapshot::permanent_card).collect::<Vec<_>>()}))}))
}
fn room_name(kind:RoomViewKind)->(&'static str,String) {
    use RoomViewKind::*;let name=match kind {Neow=>"neow",Map=>"map",Combat=>"combat",Rewards=>"rewards",CardReward=>"card_reward",Shop=>"shop",Campfire=>"campfire",Event(id)=>return ("event",id.as_str().into()),Treasure=>"treasure",CheckpointComplete=>"checkpoint_complete",BossTreasure=>"boss_treasure",BossRelicChoice=>"boss_relic_choice",BossRelicChild=>"boss_relic_child",ProceedWithoutRelic=>"proceed_without_relic",Act2Ready=>"act2_ready",Act2PrefixComplete=>"act2_prefix_complete",Act3Ready=>"act3_ready",Act3PrefixComplete=>"act3_prefix_complete",Act3BossComplete=>"act3_boss_complete",Act3Ending=>"act3_ending",Act4Ready=>"act4_ready"};(name,name.into())
}

pub(crate) fn actor_ref(actor:&Actor,combat:&str)->String {
    match actor {Actor::Player=>"player".into(),Actor::Enemy(id)=>format!("{combat}/enemy/{}",id.as_str())}
}
pub(crate) fn card_ref(card:&CardHandle,combat:&str)->String {format!("{combat}/card/{}",card.as_str())}
pub(crate) fn effects(values:&[ResolvedEffect],combat:&str)->Vec<Value> {
    values.iter().map(|e| {
        let source=match &e.source {Source::None=>None,Source::Actor(actor)=>Some(actor_ref(actor,combat)),Source::Card(card)=>Some(card_ref(card,combat)),Source::Potion{slot,id}=>Some(format!("{combat}/potion/{slot}/{}",id.as_str()))};
        let target=e.target.as_ref().map(|a|actor_ref(a,combat));
        let mut v=json!({"sequence":e.sequence,"source":source,"target":target});
        let data=match &e.kind {
            ResolvedEffectKind::Damage {incoming,absorbed_block,hp_before,hp_after,hp_lost,overkill,category}=>
                json!({"kind":"damage","description":"Resolved damage","incoming":incoming,"absorbedBlock":absorbed_block,"before":hp_before,"after":hp_after,"hpLost":hp_lost,"overkill":overkill,
                    "category":match category {DamageCategory::Normal=>"normal",DamageCategory::Orb=>"orb",DamageCategory::Thorns=>"thorns",DamageCategory::HpLoss=>"hp_loss"}}),
            ResolvedEffectKind::Heal {hp_before,hp_after,amount}=>json!({"kind":"heal","description":"Resolved healing","before":hp_before,"after":hp_after,"amount":amount}),
            ResolvedEffectKind::Block {before,after}=>json!({"kind":"block","description":"Block changed","before":before,"after":after,"field":"block"}),
            ResolvedEffectKind::Energy {before,after}=>json!({"kind":"energy","description":"Energy changed","before":before,"after":after,"field":"energy"}),
            ResolvedEffectKind::Power {id,before,after}=>json!({"kind":"power","description":format!("{} changed",id.0),"field":id.0,"before":before,"after":after}),
            ResolvedEffectKind::EnemyMove {actor,move_id}=>json!({"kind":"enemy_move","description":format!("Resolved {}",move_id.as_str()),"resolvedMove":{"actor":actor_ref(actor,combat),"id":move_id.as_str(),"name":move_id.as_str()}}),
            ResolvedEffectKind::LifeChange {actor,before,after}=>json!({"kind":"life_change","description":"Actor lifecycle changed","actor":actor_ref(actor,combat),"before":before.map(life),"after":life(*after)}),
            ResolvedEffectKind::CardMove {card,before,after}=>json!({"kind":"card_move","description":"Card changed public zone","card":card_ref(card,combat),"before":zone(*before),"after":zone(*after)}),
        };
        v.as_object_mut().unwrap().extend(data.as_object().unwrap().clone());v
    }).collect()
}
fn life(value:PublicLife)->&'static str {match value {PublicLife::Active=>"alive",PublicLife::Defeated=>"defeated",PublicLife::Replaced=>"replaced",PublicLife::Escaped=>"escaped",PublicLife::HalfDead=>"half_dead"}}
fn zone(value:PublicZone)->&'static str {match value {PublicZone::Unknown=>"unknown",PublicZone::Hand=>"hand",PublicZone::Discard=>"discard",PublicZone::Exhaust=>"exhaust",PublicZone::InPlay=>"in_play",PublicZone::Stasis=>"stasis"}}

pub(crate) fn legacy_action(action:&Action,before:&PublicView,combat:&str)->Value {
    match action {
        Action::EndTurn=>json!({"type":"end_turn","description":"End Turn","source":"player","targets":[],"card":null,"optionKey":null,"preActionOffer":null,"parentActionId":null}),
        Action::PlayCard {card_instance_id,target_id}=> {
            let card=before.hand.iter().find(|c|&c.instance_id==card_instance_id).map(|c|crate::snapshot::legacy_card(c,combat));
            json!({"type":"play_card","description":format!("Play {}",card.as_ref().and_then(|c|c["name"].as_str()).unwrap_or("card")),"source":card_ref(card_instance_id,combat),
                "targets":target_id.as_ref().map(|id|vec![actor_ref(&Actor::Enemy(id.clone()),combat)]).unwrap_or_default(),"card":card,"optionKey":null,"preActionOffer":null,"parentActionId":null})
        }
    }
}
