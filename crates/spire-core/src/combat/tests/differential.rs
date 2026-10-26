//! Bounded authored interchange. Compiled only under combat::tests, never a CLI.
use super::super::*;
use super::super::model::OwnedCard;
use super::super::rng::StreamState;
use serde_json::{json, Value};
use super::differential_decode as decode;

pub(super) const SCHEMA: &str = "spire-lab-combat-diff-v1";
pub(super) fn card(owned: &OwnedCard) -> Value {
    let c = owned.legacy().expect("legacy differential fixture owns only legacy cards");
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
pub(super) fn effect(e: &Effect) -> Value {
    let (kind,payload)=match &e.kind {
        EffectKind::Damage(DamagePayload::Fixed(n))=>("damage",json!({"amount":n})),
        EffectKind::Damage(DamagePayload::Orb(n))=>("orb_damage",json!({"amount":n})),
        EffectKind::Damage(DamagePayload::Thorns(n))=>("thorns_damage",json!({"amount":n})),
        EffectKind::Damage(DamagePayload::HpLoss(n))=>("hp_loss",json!({"amount":n})),
        EffectKind::Damage(DamagePayload::PoisonTickHpLoss{owner,amount})=>("poison_tick_hp_loss",json!({"owner":owner.as_str(),"amount":amount})),
        EffectKind::ExhaustHandCard { instance } => ("exhaust_hand_card", json!({"instance":instance.0})),
        EffectKind::Damage(DamagePayload::EnemyMove(m))=>("damage",json!({"move":m.as_str()})),
        EffectKind::Damage(DamagePayload::FullEnemyMove(m))=>("damage",json!({"full_move":m.as_str()})),
        EffectKind::GainBlock(n)=>("gain_block",json!({"amount":n})),
        EffectKind::GainEnergy(n)=>("gain_energy",json!({"amount":n})),
        EffectKind::LoseEnergy(n)=>("lose_energy",json!({"amount":n})),
        EffectKind::CreateCard { id, destination, upgrades } => ("create_card", json!({
            "card_id":id.as_str(), "destination":match destination {
                super::super::command::CardDestination::Hand => "hand",
                super::super::command::CardDestination::Draw => "draw",
                super::super::command::CardDestination::Discard => "discard",super::super::command::CardDestination::ShuffledDraw=>"shuffled_draw",super::super::command::CardDestination::BottomDraw=>"bottom_draw" },
            "upgrades":upgrades })),
        EffectKind::CopyCard { source, destination } => ("copy_card", json!({
            "source":source.as_str(), "destination":match destination {
                super::super::command::CardDestination::Hand => "hand",
                super::super::command::CardDestination::Draw => "draw",
                super::super::command::CardDestination::Discard => "discard",super::super::command::CardDestination::ShuffledDraw=>"shuffled_draw",super::super::command::CardDestination::BottomDraw=>"bottom_draw" } })),
        EffectKind::Heal(n)=> ("heal",json!({"amount":n})),
        EffectKind::IncreaseMaxHpThenHeal(n)=> ("increase_max_hp_then_heal",json!({"amount":n})),
        EffectKind::NeowsLament=>("neows_lament",json!({})),
        EffectKind::Ironclad(cards::IroncladEffect::UpgradeHand{all})=>("ironclad_upgrade_hand",json!({"all":all})),
        EffectKind::Ironclad(cards::IroncladEffect::UpgradeCard{instance})=>("ironclad_upgrade_card",json!({"instance":instance.0})),
        EffectKind::Ironclad(cards::IroncladEffect::SwordBoomerangHit{damage})=>("ironclad_sword_boomerang_hit",json!({"damage":damage})),
        EffectKind::Ironclad(cards::IroncladEffect::DoubleCurrentBlock)=>("ironclad_double_current_block",json!({})),
        EffectKind::Ironclad(cards::IroncladEffect::SpotWeakness{amount})=>("ironclad_spot_weakness",json!({"amount":amount})),
        EffectKind::Ironclad(cards::IroncladEffect::TrueGritRandomExhaust)=>("ironclad_true_grit_random_exhaust",json!({})),
        EffectKind::Ironclad(cards::IroncladEffect::ChooseTrueGritCard)=>("ironclad_choose_true_grit_card",json!({})),
        EffectKind::Ironclad(cards::IroncladEffect::ChooseHeadbuttCard)=>("ironclad_choose_headbutt_card",json!({})),
        EffectKind::Ironclad(cards::IroncladEffect::ChooseExhumeCard)=>("ironclad_choose_exhume_card",json!({})),
        EffectKind::Ironclad(cards::IroncladEffect::ChooseDualWieldCard)=>("ironclad_choose_dual_wield_card",json!({})),
        EffectKind::Ironclad(cards::IroncladEffect::WhirlwindRound{..}|cards::IroncladEffect::WhirlwindSpend|cards::IroncladEffect::HavocTop{..}|cards::IroncladEffect::HavocExtract{..}|cards::IroncladEffect::DoubleTapRepeat{..}|cards::IroncladEffect::HavocQueued{..}|cards::IroncladEffect::FinishSuppressed|cards::IroncladEffect::LimitBreakDispatch|cards::IroncladEffect::DropkickDispatch{..}|cards::IroncladEffect::RampageGrow{..}|cards::IroncladEffect::ReaperResolve{..})=>panic!("M9 v1 cannot encode M10 Ironclad work"),
        EffectKind::ExhaustBatchStart{..}|EffectKind::ExhaustBatchContinue(_)=>panic!("M9 v1 cannot encode M10 exhaust batch work"),
        EffectKind::GuardianChange(change)=>("guardian_change",json!({"change":match change {
            enemies::GuardianEffect::Defensive=>"defensive",enemies::GuardianEffect::Offensive=>"offensive",
            enemies::GuardianEffect::Reset=>"reset",enemies::GuardianEffect::LoseBlock=>"lose_block",
        }})),
        EffectKind::HexaghostBurn(gate)=>("hexaghost_burn",json!({"gate":match gate{enemies::HexaghostEffect::UpgradeBurns=>"upgrade",enemies::HexaghostEffect::InsertBurns=>"insert"}})),
        EffectKind::GroundByrd=>("ground_byrd",json!({})),
        EffectKind::SpawnEnemy(request)=>("spawn_enemy",json!({"summoner":request.summoner.as_str(),"slot":request.slot,"species":request.species.as_str()})),
        EffectKind::CompleteSpawn=>("complete_spawn",json!({})),
        EffectKind::RallyGremlins{remaining}=>("rally_gremlins",json!({"remaining":remaining})),
        EffectKind::ProtectGremlin=>("protect_gremlin",json!({})),
        EffectKind::SetEnemyMove{intent}=>("set_enemy_move",json!({"intent":intent.as_str()})),
        EffectKind::EscapeEnemy=>("escape_enemy",json!({})),
        EffectKind::BookStab{remaining}=>("book_stab",json!({"remaining":remaining})),
        EffectKind::ChampCleanse=>("champ_cleanse",json!({})),
        EffectKind::CollectorSuicide=>("collector_suicide",json!({})),
        EffectKind::AutomatonSuicide=>("automaton_suicide",json!({})),
        EffectKind::ShapeSuicide=>("shape_suicide",json!({})),
        EffectKind::ShapeDamageComplete=>("shape_damage_complete",json!({})),
        EffectKind::RevivalHeal(amount)=>("revival_heal",json!({"amount":amount})),
        EffectKind::DarklingCollapse=>("darkling_collapse",json!({})),
        EffectKind::DarklingRevive=>("darkling_revive",json!({})),
        EffectKind::DarklingsDefeat=>("darklings_defeat",json!({})),
        EffectKind::TransientFade=>("transient_fade",json!({})),
        EffectKind::TransientRecover=>("transient_recover",json!({})),
        EffectKind::ObtainParasite=>("obtain_parasite",json!({})),
        EffectKind::ReptomancerSuicide=>("reptomancer_suicide",json!({})),
        EffectKind::AwakenedCollapse=>("awakened_collapse",json!({})),
        EffectKind::AwakenedRebirth=>("awakened_rebirth",json!({})),
        EffectKind::AwakenedCanLose=>("awakened_can_lose",json!({})),
        EffectKind::TimeWarpEndTurn=>("time_warp_end_turn",json!({})),
        EffectKind::TimeEaterCleanse=>("time_eater_cleanse",json!({})),
        EffectKind::ResetMalleable=>("reset_malleable",json!({})),
        EffectKind::MawNom{remaining}=>("maw_nom",json!({"remaining":remaining})),
        EffectKind::CaptureStasis{orb}=>("capture_stasis",json!({"orb":orb.as_str()})),
        EffectKind::ReleaseStasis{orb}=>("release_stasis",json!({"orb":orb.as_str()})),
        EffectKind::ApplyPower{id,amount}=>("apply_power",json!({"power_id":id.as_str(),"amount":amount})),
        EffectKind::ReducePower{id,amount}=>("reduce_power",json!({"power_id":id.as_str(),"amount":amount})),
        EffectKind::RemovePower(id)=>("remove_power",json!({"power_id":id.as_str()})),
        EffectKind::ApplyFullPower{id,amount}=>("apply_full_power",json!({"power_id":id.0,"amount":amount})),
        EffectKind::ReduceFullPower{id,amount}=>("reduce_full_power",json!({"power_id":id.0,"amount":amount})),
        EffectKind::RemoveFullPower(id)=>("remove_full_power",json!({"power_id":id.0})),
        EffectKind::Terminal(o)=>("terminal",json!({"outcome":outcome(*o)})),
        EffectKind::Draw(n)=>("draw",json!({"amount":n})),
        EffectKind::DrawOne=>("draw_one",json!({})),
        EffectKind::AfterCardDrawn{card}=>("after_card_drawn",json!({"card":card.as_str()})),
        EffectKind::FinishCard=>("finish_card",json!({})), EffectKind::Decision=>("decision",json!({})),
        EffectKind::JuggernautRandomDamage(_)=>panic!("M9 v1 cannot encode M10 reactive work"),
        EffectKind::Ironclad(super::super::cards::IroncladEffect::InfernalBladeGenerate)=>panic!("M9 v1 cannot encode M10 generation work"),
        EffectKind::Recycle=>("recycle",json!({})), EffectKind::DiscardHand=>("discard_hand",json!({})),
        EffectKind::EndPlayerTurn=>("end_player_turn",json!({})), EffectKind::MonsterStart=>("monster_start",json!({})),
        EffectKind::EnemyMove=>("enemy_move",json!({})), EffectKind::RollMove=>("roll_move",json!({})),
        EffectKind::EndRound=>("end_round",json!({})),
        EffectKind::StartNextPlayer=>("start_next_player",json!({})),
        EffectKind::SelectCard { candidates, empty } => ("select_card", json!({"candidates":candidates.iter().map(CardHandle::as_str).collect::<Vec<_>>(),"empty":format!("{empty:?}")})),
        EffectKind::SelectCards { candidates, required, disposition, empty } => ("select_cards", json!({"candidates":candidates.iter().map(CardHandle::as_str).collect::<Vec<_>>(),"required":required,"disposition":format!("{disposition:?}"),"empty":format!("{empty:?}")})),
        EffectKind::SelectNightmareCard | EffectKind::NightmareCopy { .. } => panic!("M9 v1 cannot encode M11 Nightmare work"),
        EffectKind::SelectHandCardsAfterDraw { required, disposition, empty } => ("select_hand_after_draw", json!({"required":required,"disposition":format!("{disposition:?}"),"empty":format!("{empty:?}")})),
        EffectKind::Hook(Hook::CardAfterUse) => panic!("M9 v1 cannot encode M10 after-use Heart work"),
        EffectKind::OpenRetainSelection{..} => panic!("M9 v1 cannot encode M11 retain selection work"),
        EffectKind::Hook(hook) => ("hook", json!({"hook":format!("{hook:?}")})),
        EffectKind::PoisonTickComplete => ("poison_tick_complete", json!({})),
        EffectKind::Catalyst => panic!("M9 v1 cannot encode M11 Catalyst work"),
        EffectKind::CorruptHeartBuff => panic!("M9 v1 cannot encode M10 Corrupt Heart work"),
        EffectKind::ReplaceEnemy { parent, children } => ("replace_enemy", json!({
            "parent": parent.as_str(), "children": children.iter().map(|id| id.as_str()).collect::<Vec<_>>() })),
        EffectKind::CardContinuation(continuation) => ("card_continuation", match continuation {
            super::super::cards::DefectContinuation::GoForTheEyesWeak { amount } => json!({"go_for_the_eyes_weak":{"amount":amount}}),
            super::super::cards::DefectContinuation::BarrageStart { damage } => json!({"barrage_start":{"damage":damage}}),
            super::super::cards::DefectContinuation::BarrageNext { damage, remaining } => json!({"barrage_next":{"damage":damage,"remaining":remaining}}),
            super::super::cards::DefectContinuation::CompileDriverDraw => json!({"compile_driver_draw":true}),
        }),
        EffectKind::Character(action) => ("character", match *action {
            super::super::characters::CharacterEffect::Focus(_) => panic!("M9 v1 cannot encode M10 Focus work"),
            super::super::characters::CharacterEffect::Stance(stance) => json!({"stance":match stance {
                super::super::characters::Stance::Neutral => "neutral",
                super::super::characters::Stance::Wrath => "wrath",
                super::super::characters::Stance::Calm => "calm" }}),
            super::super::characters::CharacterEffect::Channel(orb) => json!({"channel":match orb {
                super::super::characters::OrbKind::Lightning => "lightning",
                super::super::characters::OrbKind::Frost => "frost" }}),
            super::super::characters::CharacterEffect::EvokeFront { remove } => json!({"evoke_front":{"remove":remove}}),
            super::super::characters::CharacterEffect::EvokeAt { index } => json!({"evoke_at":{"index":index}}),
            super::super::characters::CharacterEffect::EndTurn => json!({"end_turn_orbs":true}),
        }),
        EffectKind::ShieldBashFocus => panic!("M10 Shield Bash continuation is unsupported by M9 v1"),
    };
    let source=match &e.source {Source::None=>Value::Null,Source::Card(c)=>json!({"card":c.as_str()}),Source::Potion{slot,id}=>json!({"potion":{"slot":slot,"id":id.as_str()}}),Source::Actor(a)=>json!({"actor":actor(a)})};
    json!({"kind":kind,"source":source,"target":e.target.as_ref().map(actor),"payload":payload})
}
fn outcome(o: Outcome) -> &'static str { match o {Outcome::Victory=>"victory",Outcome::Death=>"death"} }
fn phase(p: Phase) -> &'static str { match p {Phase::Decision=>"decision",Phase::Selection=>"selection",Phase::PlayerAction=>"player_action",
    Phase::EnemyTurn=>"enemy_turn",Phase::RoundEnd=>"round_end",Phase::PlayerStart=>"player_start",Phase::Terminal=>"terminal"} }
fn fault(f: &FaultRecord) -> Value {
    let kind=match f.kind {FaultKind::Unsupported=>"unsupported",FaultKind::Invariant=>"invariant",FaultKind::EffectGuard=>"effect_guard",FaultKind::RngExhausted=>"rng_exhausted"};
    json!({"kind":kind,"resolved_effects":f.resolved_effects})
}
fn combat_action(a: &CombatAction) -> Value { match a {
    CombatAction::PlayCard { instance, target } => json!({"play_card":{"instance":instance,"target":target.map(|t|t.0)}}),
    CombatAction::SelectCard { instance } => json!({"select_card":{"instance":instance}}),
    CombatAction::ConfirmSelection=>json!("confirm_selection"),
    CombatAction::UsePotion { slot, target } => json!({"use_potion":{"slot":slot,"target":target.map(|id|id.0)}}),
    CombatAction::EndTurn => json!("end_turn"),
} }
fn selection_capture(s: &super::super::model::PendingSelection) -> Value {
    let parent=match &s.parent{SelectionParent::Card(card)=>json!({"card":card.as_str()}),
        SelectionParent::Potion{slot,id}=>json!({"potion":{"slot":slot,"id":id.as_str()}}),
        SelectionParent::EndTurn{..}=>panic!("M9 replay-v1 cannot encode the M11 end-turn selection parent")};
    json!({"parent":parent,"source_zone":format!("{:?}",s.source_zone),"candidates":s.candidates.iter().map(CardHandle::as_str).collect::<Vec<_>>(),
        "selected":s.selected.iter().map(CardHandle::as_str).collect::<Vec<_>>(),"minimum":s.minimum,"maximum":s.maximum,
        "disposition":format!("{:?}",s.disposition),"empty":format!("{:?}",s.empty)})
}
pub(super) fn encode_checkpoint(c: &Combat) -> Value {
    let s=&c.state; let p=&s.player;
    assert!(s.play_frames.is_empty()&&s.queued_autoplay.is_empty(),
        "M9 v1 cannot encode an active or retained M10 autoplay play frame");
    assert!(!s.enemies.iter().any(|enemy|enemy.enemy_id==ActorEnemyId::Full(crate::content::EnemyId::CorruptHeart)),
        "M9 v1 cannot encode M10 Corrupt Heart state");
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
        "outcome":s.outcome.map(outcome),"fault":s.fault.as_ref().map(fault),
        "attempted_action":s.attempted_action.as_ref().map(combat_action),
        "fault_selection":s.fault_selection.as_ref().map(selection_capture),
        "item_context":s.item_context.as_ref().map(|items| json!({
            "relics":items.relics().iter().map(|id|id.as_str()).collect::<Vec<_>>(),
            "potions":items.potions().iter().map(|id|id.map(|id|id.as_str())).collect::<Vec<_>>() })),
        "hp_before_healing":s.hp_before_healing,"hp_after_cleanup":s.hp_after_cleanup})
}

#[test]
#[should_panic(expected="M9 v1 cannot encode M10 Corrupt Heart state")]
fn m9_v1_explicitly_rejects_a_real_heart_checkpoint(){
    let combat=super::m10_heart::actual_heart();
    let _=encode_checkpoint(&combat);
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
            let expected=["content_id","phase","turn","player","enemies","piles","card_ids_issued","rng","pending","current","in_play","resolved_effects","public","outcome","fault","attempted_action","fault_selection","item_context","hp_before_healing","hp_after_cleanup"];
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

#[test]
fn m10_focus_character_work_is_explicitly_rejected_by_unchanged_m9_v1_serializer(){
    let work=Effect{source:Source::Actor(Actor::Enemy(EnemyHandle::parse("shield-0").unwrap())),
        target:Some(Actor::Player),kind:EffectKind::Character(super::super::characters::CharacterEffect::Focus(-1))};
    let rejected=std::panic::catch_unwind(||effect(&work));
    assert!(rejected.is_err(),"M9-v1 wire remains closed rather than silently encoding M10 Focus work");
}

#[test]
fn thorns_payload_keeps_a_distinct_differential_diagnostic() {
    let thorns = Effect { source: Source::Actor(Actor::Player),
        target: Some(Actor::Enemy(EnemyHandle::parse("enemy-1").unwrap())),
        kind: EffectKind::Damage(DamagePayload::Thorns(40)) };
    let encoded = effect(&thorns);
    assert_eq!(encoded["kind"], "thorns_damage");
    assert_eq!(encoded["payload"]["amount"], 40);
    assert_eq!(encoded["source"]["actor"], "player");
}


#[test]
fn max_hp_then_heal_has_a_distinct_differential_diagnostic() {
    let fruit = Effect { source: Source::Actor(Actor::Player), target: Some(Actor::Player),
        kind: EffectKind::IncreaseMaxHpThenHeal(10) };
    let encoded = effect(&fruit);
    assert_eq!(encoded["kind"], "increase_max_hp_then_heal");
    assert_eq!(encoded["payload"]["amount"], 10);
    assert_eq!(encoded["source"]["actor"], "player");
}
