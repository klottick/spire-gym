//! spire-90f: static descriptor faces equal the live card view of the same unmodified
//! card in an entered full-runtime combat, for every card colour.
use super::super::*;
use super::full_runtime::entered_combat;
use crate::content::{CardId, Character};

#[test]
fn static_faces_equal_unmodified_live_card_views_across_colours() {
    let cases = [
        (CardId::Strike, Character::Ironclad), (CardId::Defend, Character::Ironclad), (CardId::Bash, Character::Ironclad),
        (CardId::Neutralize, Character::Silent), (CardId::Survivor, Character::Silent), (CardId::Reflex, Character::Silent),
        (CardId::Zap, Character::Defect), (CardId::BallLightning, Character::Defect),
        (CardId::Eruption, Character::Watcher), (CardId::Vigilance, Character::Watcher),
        (CardId::SwiftStrike, Character::Ironclad), (CardId::Wound, Character::Ironclad),
    ];
    let mut numbered = 0;
    for (id, character) in cases {
        for upgrades in 0..=static_card_upgrade_limit(id).unwrap_or(1).min(1) {
            let face = static_card_face(id, upgrades).unwrap();
            let mut combat = entered_combat(3, 3, |_| {});
            let instance = combat.state.piles.hand[0].full().unwrap().instance;
            *combat.state.piles.hand[0].full_mut().unwrap() = complete_live_card(id, instance, upgrades).unwrap();
            combat.state.character = characters::CharacterState::for_character(character);
            combat.state.item_context = Some(crate::items::CombatItemContext::new(vec![], vec![]).with_complete_cards());
            let view = combat.public_full().unwrap();
            let live = view.hand.iter().find(|card| card.instance == instance).unwrap();
            let label = format!("{}+{upgrades}", id.as_str());
            match face.cost {
                StaticCost::Energy(cost) => assert_eq!(live.effective_cost, Some(cost), "{label}"),
                StaticCost::Unplayable | StaticCost::X => assert_eq!(live.effective_cost, None, "{label}"),
                StaticCost::Unresolved => panic!("{label} unresolved"),
            }
            if let Some(values) = &live.values {
                assert_eq!((face.base_damage, face.base_block, face.magic_number),
                    (values.base_damage, values.base_block, values.magic_number), "{label}");
                assert_eq!(face.cost == StaticCost::Unplayable, values.cost_kind == CardCostKind::Unplayable, "{label}");
                numbered += usize::from(values.base_damage.or(values.base_block).is_some());
            }
        }
    }
    assert!(numbered >= 14, "public numbers projected for {numbered} faces");
}
