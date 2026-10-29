//! Semantic output for already chosen public actions; no parser or simulator.
use serde_json::{Value,json};
use spire_core::{combat::CombatAction,run::GameAction};
/// Output uses the shared native replay verb/field convention. This serializer
/// reads an already-chosen public action; it does not parse or execute rules.
pub fn combat_action_value(action:&CombatAction)->Value{match action{
    CombatAction::PlayCard{instance,target}=>json!({"verb":"combat.play_card","instance":instance,"target":target.map(|t|t.0)}),
    CombatAction::SelectCard{instance}=>json!({"verb":"combat.select_card","instance":instance}),
    CombatAction::DeselectCard{instance}=>json!({"verb":"combat.deselect_card","instance":instance}),
    CombatAction::ChooseStance{stance}=>json!({"verb":"combat.choose_stance","stance":match stance{spire_core::combat::Stance::Neutral=>"neutral",spire_core::combat::Stance::Wrath=>"wrath",spire_core::combat::Stance::Calm=>"calm",spire_core::combat::Stance::Divinity=>"divinity"}}),
    CombatAction::UsePotion{slot,target}=>json!({"verb":"combat.use_potion","slot":slot,"target":target.map(|t|t.0)}),
    CombatAction::EndTurn=>json!({"verb":"combat.end_turn"}),CombatAction::ConfirmSelection=>json!({"verb":"combat.confirm_selection"}),
    CombatAction::ChooseCardOffer{index}=>json!({"verb":"combat.choose_card_offer","index":index}),CombatAction::SkipCardOffer=>json!({"verb":"combat.skip_card_offer"}),
}}

/// Run actions use the existing native replay verb/field convention.
pub fn action_value(action:&GameAction)->Value{match action{
    GameAction::Move(id)=>json!({"verb":"move","node":id.0}),
    GameAction::Choose(id)=>json!({"verb":"choose","choice":id.0}),
    GameAction::Select(id)=>json!({"verb":"select","selection":id.0}),
    GameAction::Confirm=>json!({"verb":"confirm"}),GameAction::Cancel=>json!({"verb":"cancel"}),
    GameAction::UsePotion{slot,target}=>json!({"verb":"use_potion","slot":slot,"target":target.map(|t|t.0)}),
    GameAction::DiscardPotion{slot}=>json!({"verb":"discard_potion","slot":slot}),
    GameAction::Combat(action)=>combat_action_value(action),
}}
