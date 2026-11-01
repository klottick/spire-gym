//! Fixed, default-off M26 complete-card starts (plan Task M3). Resources are supplied and
//! checked by `CompleteCombatStart`; the Run's card admission is `Complete`. No arbitrary
//! state importer and no public supplied-run API; callers label results "Supplied played".
use super::super::*;
use crate::{combat::rng::Partition,content::{Character,EncounterId}};

fn random()->RunRng{let mut root=[0;32];root[..8].copy_from_slice(&26u64.to_le_bytes());RunRng::new(root,Partition::Test,0)}
fn heart_scenario(character:Character)->RoomScenario{
    match character{Character::Ironclad=>RoomScenario::M10IroncladHeart,Character::Silent=>RoomScenario::M15SilentHeart,
        Character::Defect=>RoomScenario::M20DefectHeart,Character::Watcher=>RoomScenario::M25WatcherHeart}
}

/// The character's complete Heart scenario with `start` as its resources (checked by
/// `CompleteCombatStart`) and `CardAdmission::Complete`; no combat, zero accepted actions.
pub fn complete_run(character:Character,start:AuthoredRunStart)->Result<Run,RunError>{
    let CompleteCombatStart::Supplied(resources)=CompleteCombatStart::Supplied(start).normalized(character)?;
    let next=resources.deck.iter().map(|c|c.instance).max().unwrap().checked_add(1).ok_or(RunError::Unsupported("permanent identity exhausted"))?;
    let mut profile=CreationProfile::experienced();profile.neow_eligible=false;
    let mut run=Run::new_room_scenario(RunConfig{character,ascension:0},profile,random(),heart_scenario(character))?;
    let s=&mut run.state;
    s.hp=resources.hp;s.max_hp=resources.max_hp;s.gold=resources.gold;s.deck=resources.deck;s.relics=resources.relics;
    s.potions=resources.potions;s.relic_counters=resources.relic_counters;s.next_instance=next;s.card_admission=CardAdmission::Complete;
    run.candidates.clear();Ok(run)
}

/// The same admission, then one combat entered through `Run::from_validated_combat_resources`
/// and `Run::enter_owned` (via the training-encounter owner).
pub fn complete_combat(character:Character,start:AuthoredRunStart,encounter:EncounterId)->Result<Run,RunError>{
    let mut run=Run::prepare_complete_combat(CompleteCombatStart::Supplied(start),character,encounter,random())?;
    run.enter_training_combat(encounter)?;Ok(run)
}
