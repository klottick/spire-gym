//! Bounded authored interchange. Compiled only under combat::tests, never a CLI.
use super::super::*;
use super::super::rng::StreamState;
use serde_json::{json, Value};
use super::differential_decode as decode;

pub(super) const SCHEMA: &str = "spire-lab-combat-diff-v1";
pub(super) fn card(c: &CardInstance) -> Value {
    json!({"instance_id":c.instance_id.as_str(),"card_id":c.card_id.as_str(),"upgraded":c.upgraded})
}
fn powers(p: &[PowerEntry]) -> Value {
    p.iter().map(|p| json!({"id":p.id.as_str(),"amount":p.state.amount,
        "skip_first":p.state.skip_first,"monster_source_grace":p.state.monster_source_grace})).collect()
}
fn stream(s: &StreamState) -> Value {
    json!({"domain":s.id().domain().as_str(),"scope":s.id().scope(),
        "key_hex":s.key().iter().map(|b| format!("{b:02x}")).collect::<String>(),"word_index":s.word_index().to_string()})
}
fn actor(a: &Actor) -> &str { match a { Actor::Player=>"player",Actor::Enemy(e)=>e.as_str() } }
fn effect(e: &Effect) -> Value {
    let (kind,payload)=match &e.kind {
        EffectKind::Damage(DamagePayload::Fixed(n))=>("damage",json!({"amount":n})),
        EffectKind::Damage(DamagePayload::EnemyMove(m))=>("damage",json!({"move":m.as_str()})),
        EffectKind::GainBlock(n)=>("gain_block",json!({"amount":n})),
        EffectKind::ApplyPower{id,amount}=>("apply_power",json!({"power_id":id.as_str(),"amount":amount})),
        EffectKind::ReducePower{id,amount}=>("reduce_power",json!({"power_id":id.as_str(),"amount":amount})),
        EffectKind::RemovePower(id)=>("remove_power",json!({"power_id":id.as_str()})),
        EffectKind::Terminal(o)=>("terminal",json!({"outcome":outcome(*o)})),
        EffectKind::Draw(n)=>("draw",json!({"amount":n})),
        EffectKind::FinishCard=>("finish_card",json!({})), EffectKind::Decision=>("decision",json!({})),
        EffectKind::Recycle=>("recycle",json!({})), EffectKind::DiscardHand=>("discard_hand",json!({})),
        EffectKind::EndPlayerTurn=>("end_player_turn",json!({})), EffectKind::MonsterStart=>("monster_start",json!({})),
        EffectKind::EnemyMove=>("enemy_move",json!({})), EffectKind::RollMove=>("roll_move",json!({})),
        EffectKind::EndRound=>("end_round",json!({})),
    };
    let source=match &e.source {Source::None=>Value::Null,Source::Card(c)=>json!({"card":c.as_str()}),Source::Actor(a)=>json!({"actor":actor(a)})};
    json!({"kind":kind,"source":source,"target":e.target.as_ref().map(actor),"payload":payload})
}
fn outcome(o: Outcome) -> &'static str { match o {Outcome::Victory=>"victory",Outcome::Death=>"death"} }
fn phase(p: Phase) -> &'static str { match p {Phase::Decision=>"decision",Phase::PlayerAction=>"player_action",
    Phase::EnemyTurn=>"enemy_turn",Phase::RoundEnd=>"round_end",Phase::PlayerStart=>"player_start",Phase::Terminal=>"terminal"} }
fn fault(f: &FaultRecord) -> Value {
    let kind=match f.kind {FaultKind::Unsupported=>"unsupported",FaultKind::Invariant=>"invariant",FaultKind::EffectGuard=>"effect_guard",FaultKind::RngExhausted=>"rng_exhausted"};
    json!({"kind":kind,"resolved_effects":f.resolved_effects})
}
pub(super) fn encode_checkpoint(c: &Combat) -> Value {
    let s=&c.state; let p=&s.player;
    let enemies:Vec<_>=s.enemies.iter().map(|e| {
        let damage:serde_json::Map<String,Value>=e.damage_entries.iter().map(|(id,d)| (id.as_str().into(),json!({"base":d.base,"output":d.output}))).collect();
        json!({"instance_id":e.instance_id.as_str(),"enemy_id":e.enemy_id.as_str(),"hp":e.hp,"max_hp":e.max_hp,"block":e.block,"powers":powers(&e.powers),
            "intent_id":e.intent_id.as_str(),"move_history":e.move_history.iter().map(|m|m.as_str()).collect::<Vec<_>>(),
            "selected_move":e.selected_move.as_str(),"damage_entries":damage,"intent_damage":e.intent_damage})
    }).collect();
    json!({"content_id":s.content_id,"phase":phase(s.phase),"turn":s.turn,
        "player":{"hp":p.hp,"max_hp":p.max_hp,"block":p.block,"energy":p.energy,"max_energy":p.max_energy,"draw_per_turn":p.draw_per_turn,"powers":powers(&p.powers)},
        "enemies":enemies,"piles":{"hand":s.piles.hand.iter().map(card).collect::<Vec<_>>(),"draw":s.piles.draw.iter().map(card).collect::<Vec<_>>(),
            "discard":s.piles.discard.iter().map(card).collect::<Vec<_>>(),"exhaust":s.piles.exhaust.iter().map(card).collect::<Vec<_>>()},
        "card_ids_issued":s.card_ids_issued.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
        "rng":{"contract_id":s.rng.contract_id,"shuffle":stream(&s.rng.shuffle),"enemy_ai":stream(&s.rng.enemy_ai),"enemy_instance_id":s.rng.enemy_instance_id.as_str()},
        "pending":s.pending.iter().map(effect).collect::<Vec<_>>(),"current":s.current.as_ref().map(effect),"in_play":s.in_play.as_ref().map(card),
        "resolved_effects":s.resolved_effects,"public":{"revealed_order":s.public.revealed_order.iter().map(|id|id.as_str()).collect::<Vec<_>>()},
        "outcome":s.outcome.map(outcome),"fault":s.fault.as_ref().map(fault),"hp_before_healing":s.hp_before_healing,"hp_after_cleanup":s.hp_after_cleanup})
}
pub(super) fn action(a: &Action) -> Value { match a {
    Action::EndTurn=>json!({"kind":"end_turn"}),
    Action::PlayCard{card_instance_id,target_id}=>json!({"kind":"play_card","card_instance_id":card_instance_id.as_str(),"target_id":target_id.as_ref().map(|t|t.as_str())})
} }
fn public(c: &Combat) -> Value {
    let v=c.public_view().unwrap(); let p=&v.player;
    let pp=|powers:&Vec<(PowerId,i32)>| { let mut pairs:Vec<_>=powers.iter().map(|(id,n)|(id.as_str(),*n)).collect();pairs.sort();json!(pairs) };
    let cv=|cards:&Vec<CardView>| cards.iter().map(|c|json!({"instance_id":c.instance_id.as_str(),"card_id":c.card_id.as_str(),"upgraded":c.upgraded})).collect::<Vec<_>>();
    json!({"content_id":v.content_id,"turn":v.turn,"player":{"hp":p.hp,"max_hp":p.max_hp,"block":p.block,"energy":p.energy,"max_energy":p.max_energy,"powers":pp(&p.powers)},
        "enemies":v.enemies.iter().map(|e|json!({"instance_id":e.instance_id.as_str(),"enemy_id":e.enemy_id.as_str(),"hp":e.hp,"max_hp":e.max_hp,"block":e.block,
            "powers":pp(&e.powers),"intent_id":e.intent_id.as_str(),"intent_damage":e.intent_damage})).collect::<Vec<_>>(),
        "hand":cv(&v.hand),"draw_count":v.draw_count,"discard":cv(&v.discard),"exhaust":cv(&v.exhaust),
        "revealed_order":v.revealed_order.iter().map(|id|id.as_str()).collect::<Vec<_>>(),"outcome":v.outcome.map(outcome),"hp_before_healing":v.hp_before_healing,"hp_after_cleanup":v.hp_after_cleanup})
}
fn record(c: &Combat,index:usize,rejection:Option<&str>) -> Value {
    let status=if rejection.is_some(){"rejected"}else if c.state.fault.is_some(){"fault"}else if c.state.outcome.is_some(){"terminal"}else{"decision"};
    let (view,legal)=if c.state.fault.is_some(){(Value::Null,Value::Null)}else{(public(c),c.legal_actions().unwrap().iter().map(action).collect())};
    json!({"index":index,"status":status,"rejection":rejection,"state":encode_checkpoint(c),"public":view,"legal":legal})
}
fn response(request: &Value) -> Value {
    let case_id=request.get("case_id").and_then(Value::as_str).unwrap_or("");
    let mut result=json!({"schema":SCHEMA,"case_id":case_id,"entry_error":null,"checkpoints":[]});
    let decoded=decode::request(request);
    let (scenario,actions)=match decoded {Ok(v)=>v,Err(e)=>{result["entry_error"]=json!(e);return result;}};
    let mut c=match Combat::new(&scenario) {Ok(c)=>c,Err(e)=>{result["entry_error"]=json!(match e {InputError::Invariant(_)=>"invariant",InputError::Unsupported(_)=>"unsupported"});return result;}};
    let mut points=vec![record(&c,0,None)];
    for (i,raw) in actions.iter().enumerate() {
        // Restore a detached safe checkpoint before every action and compare full results.
        let saved=c.snapshot(); let baseline=encode_checkpoint(&c);
        let mut replay=if c.state.fault.is_none(){Some(Combat::restore(&saved).unwrap())}else{None};
        let rejection=match decode::action(raw) { Err(e)=>Some(e),Ok(a)=>match c.step(&a) {
            Ok(_)=>None,Err(e)=>Some(match e {StepError::InvalidAction=>"invalid_action",StepError::NotReady=>"not_ready",StepError::Terminal=>"terminal",StepError::Faulted=>"faulted"})}};
        if let Some(r)=replay.as_mut() {
            if let Ok(a)=decode::action(raw) {let _=r.step(&a);}
            assert_eq!(encode_checkpoint(&c),encode_checkpoint(r));
            assert_eq!(record(&c,i+1,rejection),record(r,i+1,rejection));
            assert_eq!(encode_checkpoint(&Combat::restore(&saved).unwrap()),baseline);
        }
        points.push(record(&c,i+1,rejection));
    }
    result["checkpoints"]=json!(points);result
}

#[test]
fn protocol() {
    match (std::env::var_os("SPIRE_COMBAT_DIFF_REQUEST"),std::env::var_os("SPIRE_COMBAT_DIFF_RESPONSE")) {
        (None,None)=>{let c=Combat::new(&super::whole_combat::scenario(EnemyKind::Cultist)).unwrap();
            let r=record(&c,0,None); assert_eq!(r["state"]["resolved_effects"],0);
            let expected=["content_id","phase","turn","player","enemies","piles","card_ids_issued","rng","pending","current","in_play","resolved_effects","public","outcome","fault","hp_before_healing","hp_after_cleanup"];
            assert_eq!(r["state"].as_object().unwrap().keys().map(String::as_str).collect::<std::collections::BTreeSet<_>>(),expected.into_iter().collect());
            assert_eq!(r["public"]["player"]["hp"],40); assert_eq!(r["legal"].as_array().unwrap().len(),4);},
        (Some(input),Some(output))=>{
            let input=std::path::PathBuf::from(input); let output=std::path::PathBuf::from(output);
            assert!(input.is_absolute()&&output.is_absolute());
            let bytes=std::fs::read(input).unwrap();
            // serde_json rejects non-scalar strings (including escaped lone surrogates).
            let r=match serde_json::from_slice::<Value>(&bytes) {Ok(v)=>response(&v),Err(_)=>{
                // Decode only the envelope identity, ignoring malformed scenario strings.
                // Never use lossy Unicode replacement or a text-search identity guess.
                #[derive(serde::Deserialize)]
                struct Identity { case_id: String }
                let id=serde_json::from_slice::<Identity>(&bytes).map(|v|v.case_id).unwrap_or_default();
                json!({"schema":SCHEMA,"case_id":id,"entry_error":"interchange","checkpoints":[]})}};
            let file=std::fs::OpenOptions::new().write(true).create_new(true).open(output).unwrap();serde_json::to_writer(file,&r).unwrap();
        },
        _=>panic!("both interchange paths are required"),
    }
}
