//! Strict test-only decoding; no serde on product/snapshot state.
use super::super::*;
use super::super::rng::{Domain,StreamId,StreamState};
use serde_json::Value;
type R<T> = Result<T,&'static str>;
const BAD: &str="interchange";
fn fields<'a>(v:&'a Value,names:&[&str])->R<&'a Value> {
    let o=v.as_object().ok_or(BAD)?;
    if o.len()!=names.len() || names.iter().any(|n|!o.contains_key(*n)){return Err(BAD);} Ok(v)
}
fn text(v:&Value)->R<&str>{v.as_str().ok_or(BAD)}
fn integer(v:&Value)->R<i32>{v.as_i64().and_then(|n|i32::try_from(n).ok()).ok_or(BAD)}
fn boolean(v:&Value)->R<bool>{v.as_bool().ok_or(BAD)}
fn list(v:&Value)->R<&Vec<Value>>{v.as_array().ok_or(BAD)}
fn finite<'a>(v:&'a Value,allowed:&[&str])->R<&'a str>{
    let s=text(v)?;
    if s.is_empty() || !s.as_bytes()[0].is_ascii_lowercase() || s.split('_').any(|part|part.is_empty()||!part.bytes().all(|b|b.is_ascii_lowercase()||b.is_ascii_digit())) {return Err(BAD);}
    if !allowed.contains(&s){return Err("unsupported");} Ok(s)
}
fn enemy(v:&Value)->R<EnemyKind>{Ok(match finite(v,&["cultist","jaw_worm"])? {"cultist"=>EnemyKind::Cultist,_=>EnemyKind::JawWorm})}
fn movement(v:&Value)->R<MoveId>{Ok(match finite(v,&["incantation","attack","chomp","thrash","bellow"])? {
    "incantation"=>MoveId::Incantation,"attack"=>MoveId::Attack,"chomp"=>MoveId::Chomp,"thrash"=>MoveId::Thrash,_=>MoveId::Bellow})}
fn power(v:&Value)->R<PowerId>{Ok(match finite(v,&["strength","ritual","vulnerable"])?{"strength"=>PowerId::Strength,"ritual"=>PowerId::Ritual,_=>PowerId::Vulnerable})}
fn handle(v:&Value)->R<CardHandle>{CardHandle::parse(text(v)?).map_err(|_|BAD)}
fn owner(v:&Value)->R<EnemyHandle>{EnemyHandle::parse(text(v)?).map_err(|_|"invariant")}
fn cards(v:&Value)->R<Vec<CardInstance>>{list(v)?.iter().map(|c|{
    fields(c,&["instance_id","card_id","upgraded"])?;
    Ok(CardInstance{instance_id:handle(&c["instance_id"])?,card_id:match finite(&c["card_id"],&["strike","defend","bash"])?{"strike"=>CardKind::Strike,"defend"=>CardKind::Defend,_=>CardKind::Bash},upgraded:boolean(&c["upgraded"])?})
}).collect()}
fn powers(v:&Value)->R<Vec<PowerEntry>>{
    let mut entries=vec![];
    for p in list(v)? {fields(p,&["id","amount","skip_first","monster_source_grace"])?;
        let id=power(&p["id"])?;
        if entries.iter().any(|p:&PowerEntry|p.id==id){return Err("invariant");}
        entries.push(PowerEntry{id,state:PowerState{amount:integer(&p["amount"])?,skip_first:boolean(&p["skip_first"])?,monster_source_grace:boolean(&p["monster_source_grace"])?}});
    } Ok(entries)
}
fn stream(v:&Value)->R<StreamState>{
    fields(v,&["domain","scope","key_hex","word_index"])?;
    let domain=match text(&v["domain"])? {"combat.shuffle"=>Domain::CombatShuffle,"combat.enemy.ai"=>Domain::CombatEnemyAi,"combat.enemy.hp"=>Domain::CombatEnemyHp,_=>return Err(BAD)};
    let scope=list(&v["scope"])?.iter().map(|s|text(s).map(str::to_owned)).collect::<R<Vec<_>>>()?;
    let hex=text(&v["key_hex"])?;
    if hex.len()!=64 || !hex.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b)){return Err(BAD);}
    let mut key=[0;32]; for (i,b) in key.iter_mut().enumerate(){*b=u8::from_str_radix(&hex[i*2..i*2+2],16).map_err(|_|BAD)?;}
    let cursor=text(&v["word_index"])?;
    if cursor.is_empty() || (cursor.len()>1&&cursor.starts_with('0')) || !cursor.bytes().all(|b|b.is_ascii_digit()){return Err(BAD);}
    let cursor=cursor.parse::<u128>().map_err(|_|BAD)?; if cursor>1u128<<64{return Err(BAD);}
    let id=StreamId::new(domain,scope).map_err(|_|"invariant")?;
    StreamState::new(id,key,cursor).map_err(|_|"invariant")
}
pub(super) fn request(v:&Value)->R<(Scenario,&Vec<Value>)>{
    fields(v,&["schema","case_id","scenario","actions"])?;
    if text(&v["schema"])?!=super::differential::SCHEMA || text(&v["case_id"])?.is_empty(){return Err(BAD);}
    let actions=list(&v["actions"])?; let s=&v["scenario"];
    fields(s,&["encounter_id","player","enemies","piles","rng"])?;
    let p=&s["player"]; fields(p,&["hp","max_hp","block","energy","max_energy","draw_per_turn","powers"])?;
    let player=PlayerState{hp:integer(&p["hp"])?,max_hp:integer(&p["max_hp"])?,block:integer(&p["block"])?,energy:integer(&p["energy"])?,max_energy:integer(&p["max_energy"])?,draw_per_turn:integer(&p["draw_per_turn"])?,powers:powers(&p["powers"])?};
    let enemies=list(&s["enemies"])?.iter().map(|e|{
        fields(e,&["instance_id","enemy_id","hp","max_hp","block","powers","intent_id","move_history"])?;
        Ok(EnemySetup{instance_id:owner(&e["instance_id"])?,enemy_id:enemy(&e["enemy_id"])?,hp:integer(&e["hp"])?,max_hp:integer(&e["max_hp"])?,block:integer(&e["block"])?,powers:powers(&e["powers"])?,intent_id:movement(&e["intent_id"])?,move_history:list(&e["move_history"])?.iter().map(movement).collect::<R<Vec<_>>>()?})
    }).collect::<R<Vec<_>>>()?;
    let p=&s["piles"];fields(p,&["hand","draw","discard","exhaust"])?;
    let piles=Piles{hand:cards(&p["hand"])?,draw:cards(&p["draw"])?,discard:cards(&p["discard"])?,exhaust:cards(&p["exhaust"])?};
    let r=&s["rng"];fields(r,&["contract_id","enemy_instance_id","shuffle","enemy_ai"])?;
    let rng=CombatRng{contract_id:text(&r["contract_id"])?.into(),enemy_instance_id:owner(&r["enemy_instance_id"])?,shuffle:stream(&r["shuffle"])?,enemy_ai:stream(&r["enemy_ai"])?};
    Ok((Scenario{encounter_id:enemy(&s["encounter_id"])?,player,enemies,piles,rng},actions))
}
pub(super) fn action(v:&Value)->R<Action>{
    let kind=finite(v.get("kind").ok_or(BAD)?,&["play_card","end_turn"])?;
    if kind=="end_turn"{fields(v,&["kind"])?;return Ok(Action::EndTurn);}
    fields(v,&["kind","card_instance_id","target_id"])?;
    let card_instance_id=handle(&v["card_instance_id"])?;
    let target_id=if v["target_id"].is_null(){None}else{
        Some(EnemyHandle::parse(text(&v["target_id"])?).map_err(|_|"invalid_action")?)};
    Ok(Action::PlayCard{card_instance_id,target_id})
}
