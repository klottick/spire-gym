//! spire-oto.27: MonsterRoomElite.applyEmeraldEliteBuff scales with the act (source-check-monsterturn-v3
//! Q6): Strength act+1, Metallicize 2·act+2, RegenerateMonsterPower (ID Regenerate) 2·act+1.
//! The engine's burning regeneration is `regenerate`, the Awakened One's power; `regeneration`
//! is the player's RegenPower only.
use super::super::*;
use super::m5_elites::{amount, entry, hit};
use crate::content::EncounterId;
use enemies::BurningBuff as B;

#[test]
fn fuzzfix_d_burning_elite_buff_scales_with_the_act() {
    for (act, strength, metallicize, regenerate) in [(1, 2, 4, 3), (2, 3, 6, 5), (3, 4, 8, 7)] {
        for (buff, id, want) in [(B::Strength, "strength", strength), (B::Metallicize, "metallicize", metallicize), (B::Regeneration, "regenerate", regenerate)] {
            let mut e = entry(EncounterId::ThreeSentries);
            enemies::apply_burning_buff(&mut e, act, buff).unwrap();
            let c = Combat::enter(e).unwrap();
            for slot in 0..3 {
                assert_eq!(amount(&c, slot, id), want, "act {act} {buff:?} slot {slot}");
                assert_eq!(amount(&c, slot, "regeneration"), 0, "act {act} {buff:?}: never the player's RegenPower id");
            }
        }
    }
}

#[test]
fn fuzzfix_d_act_three_burning_regenerate_heals_seven_at_end_turn() {
    let mut e = entry(EncounterId::ThreeSentries);
    enemies::apply_burning_buff(&mut e, 3, B::Regeneration).unwrap();
    let mut c = Combat::enter(e).unwrap();
    let max = c.state.enemies[1].max_hp;
    hit(&mut c, 1, 10);
    c.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(c.state.enemies[1].hp, max - 3);
    assert_eq!(amount(&c, 1, "regenerate"), 7, "RegenerateMonsterPower keeps its amount");
}
