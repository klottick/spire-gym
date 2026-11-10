//! M34s-a `standard-combat-v1` (design §4.4): one Standard combat entered from owned resources through the shared
//! Run owner (`from_validated_combat_resources`, `enter_owned`, the shared resolver). The Run is in the settlement-free
//! combat scope: the episode ends at the combat's own terminal; `settle_combat`, rewards and Run-level settlement
//! effects never run.
use super::*;
use crate::combat::enemies::{EncounterKind, A0_ENCOUNTER_SETUPS};
use crate::content::{Character, EncounterId};

/// One owned Standard combat start. Resources are already owned: acquisition effects are not re-applied.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StandardCombatStart {
    pub character:Character, pub ascension:u8, pub encounter:EncounterId,
    /// Elite encounters only: the Emerald-key buff the original rolls at entry, named by the caller.
    pub burning:Option<BurningEliteBuff>,
    pub resources:AuthoredRunStart,
}

/// The combat's own terminal (design §4.4): its outcome, the HP before and after the combat's end-of-combat hooks
/// (Burning Blood, Black Blood, Repair) and whether every enemy escaped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CombatScopeTerminal { pub outcome:crate::combat::Outcome, pub hp_before_healing:u32, pub hp_after_cleanup:u32, pub all_enemies_escaped:bool }

/// The Standard setup row of `encounter`: its setup act and kind (every row of the shared setup table).
pub fn standard_combat_encounter(encounter:EncounterId)->Option<(u8,EncounterKind)>{
    A0_ENCOUNTER_SETUPS.iter().find(|(id,_,_)|*id==encounter).map(|(_,act,kind)|(*act,*kind))
}

impl Run {
    /// Validate and own the resources of one Standard combat (complete card and item admission at `ascension`).
    pub fn prepare_standard_combat(start:StandardCombatStart,rng:RunRng)->Result<Self,RunError>{
        if start.ascension>STANDARD_MAX_ASCENSION{return Err(RunError::InvalidAscension(start.ascension))}
        let (act,kind)=standard_combat_encounter(start.encounter).ok_or(RunError::Unsupported("encounter outside the Standard setup table"))?;
        if start.burning.is_some()&&kind!=EncounterKind::Elite{return Err(RunError::Unsupported("burning requires an elite encounter"))}
        let CompleteCombatStart::Supplied(resources)=CompleteCombatStart::Supplied(start.resources).normalized_standard(start.character,start.ascension)?;
        let next=resources.deck.iter().map(|c|c.instance).max().unwrap().checked_add(1).ok_or(RunError::Unsupported("permanent identity exhausted"))?;
        let mut run=Self::from_validated_combat_resources(RunConfig{character:start.character,ascension:start.ascension},resources,rng,next);
        run.state.authored_combat=false;run.state.act=act;run.state.room="StandardCombat";
        run.state.card_admission=CardAdmission::Complete;run.state.item_admission=ItemAdmission::Complete;
        run.state.relic_pools=Some(crate::items::relic_pools::build(&run.state,&[])?);
        run.state.combat_scope=Some(CombatScope{encounter:start.encounter,burning:start.burning});
        Ok(run)
    }
    /// Enter the prepared combat once, through `enter_owned` (an encounter decided at entry stops at its terminal).
    pub fn enter_standard_combat(&mut self)->Result<RunTransition,RunError>{
        let scope=self.state.combat_scope.ok_or(RunError::InvalidAction)?;
        if self.state.combat_progress.entered!=0{return Err(RunError::InvalidAction)}
        let (_,kind)=standard_combat_encounter(scope.encounter).ok_or(RunError::InvalidAction)?;
        let kind=match kind{EncounterKind::Monster=>MapNodeKind::Monster,EncounterKind::Elite=>MapNodeKind::Elite,EncounterKind::Boss=>MapNodeKind::Boss};
        self.enter_owned(scope.encounter,CombatOrigin::Training{kind})
    }
    /// Fuzz oracle only (never public): the live combat, for the a6 draw-knowledge support check.
    #[cfg(any(test, feature = "fuzz-diagnostics"))]
    pub fn fuzz_combat(&self)->Option<&crate::combat::Combat>{self.state.combat.as_ref()}
    /// The combat scope's terminal, once the combat has an outcome; `None` before, and outside a combat scope.
    pub fn combat_scope_terminal(&self)->Option<CombatScopeTerminal>{
        self.state.combat_scope?;
        let combat=self.state.combat.as_ref()?;let view=combat.public_full().ok()?;
        Some(CombatScopeTerminal{outcome:view.outcome?,hp_before_healing:u32::try_from(view.hp_before_healing?).ok()?,
            hp_after_cleanup:u32::try_from(view.hp_after_cleanup?).ok()?,all_enemies_escaped:combat.all_enemies_escaped()})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::{rng::Partition, Outcome};
    use crate::content::{CardId, RelicId};

    fn start(hp:u32,relics:Vec<RelicId>)->StandardCombatStart{
        let deck=(1..=10).map(|instance|CardInstance{id:CardId::Bludgeon,instance,upgrades:1,misc:0}).collect();
        StandardCombatStart{character:Character::Ironclad,ascension:0,encounter:EncounterId::Cultist,burning:None,
            resources:AuthoredRunStart{hp,max_hp:80,gold:7,deck,relics,potions:vec![None;3],relic_counters:Default::default()}}
    }

    /// Design §4.4: the combat scope stops at the combat's own terminal: no settlement, rewards, Run terminal or
    /// Run-level random draw; the combat-end hook (Burning Blood) has applied, the settlement relic has not.
    #[test]
    fn combat_scope_stops_at_the_combat_terminal(){
        for (relics,healed) in [(vec![RelicId::BurningBlood],6),(vec![RelicId::MeatOnTheBone],0)]{
            let mut run=Run::prepare_standard_combat(start(30,relics),RunRng::new([3;32],Partition::Test,0)).unwrap();
            let deck=run.state.deck.clone();
            run.enter_standard_combat().unwrap();
            assert!(run.enter_standard_combat().is_err(),"one entry per scope");
            let rng=run.state.rng.clone();
            while run.combat_scope_terminal().is_none(){
                let d=run.decision().unwrap();
                let play=d.candidates.iter().find(|c|matches!(c.action,GameAction::Combat(crate::combat::CombatAction::PlayCard{..}))).unwrap_or(&d.candidates[0]).action.clone();
                let t=run.step(&play).unwrap();
                assert_eq!((t.reward,t.terminal),(0,None));
            }
            let t=run.combat_scope_terminal().unwrap();
            assert_eq!((t.outcome,t.hp_after_cleanup,t.all_enemies_escaped),(Outcome::Victory,(t.hp_before_healing+healed).min(80),false));
            assert_eq!(run.state.rng,rng,"no Run RNG draw after entry");
            assert_eq!((run.state.combat_progress.settled,run.state.last_combat_checkpoint.is_none(),run.state.terminal,run.state.deck.clone()),(0,true,None,deck));
            assert!(run.state.combat.is_some()&&run.decision().unwrap().candidates.is_empty());
        }
    }

    /// Final review M7 (PM ruling): only profiles that publish public knowledge maintain it — the combat scope
    /// (`standard-combat-v1`) and Standard Runs; Gym66 and captured profiles never do.
    #[test]
    fn public_knowledge_is_maintained_only_where_published(){
        let rng=||RunRng::new([5;32],Partition::Test,0);
        let mut scope=Run::prepare_standard_combat(start(70,vec![]),rng()).unwrap();scope.enter_standard_combat().unwrap();
        assert!(scope.state.combat.as_ref().unwrap().maintains_public_knowledge());
        assert!(scope.state.combat.as_ref().unwrap().draw_knowledge().is_ok());
        let encounter=EncounterId::Cultist;
        let entry=CombatTrainingEntry{encounter,hp:80,max_hp:80,deck:TRAINING_CARDS.iter().take(10).map(|c|(*c,0)).collect(),relics:vec![],potions:vec![None;3],neows_lament_remaining:None};
        let mut gym66=Run::prepare_combat_training_v2_with_gold(entry,0,rng()).unwrap();gym66.enter_training_combat(encounter).unwrap();
        let combat=gym66.state.combat.as_ref().unwrap();
        assert!(!combat.maintains_public_knowledge());
        assert!(combat.draw_knowledge().is_err()&&combat.preroll_sets().is_empty(),"no knowledge, never stale");
        let mut standard=Run::new_room_scenario(RunConfig{character:Character::Silent,ascension:0},
            CreationProfile{neow_eligible:false,..CreationProfile::experienced()},rng(),super::rooms::RoomScenario::Standard).unwrap();
        let mut m10=Run::new_room_scenario(RunConfig{character:Character::Ironclad,ascension:0},
            CreationProfile{neow_eligible:false,..CreationProfile::experienced()},rng(),super::rooms::RoomScenario::M10IroncladHeart).unwrap();
        for (run,maintained) in [(&mut standard,true),(&mut m10,false)]{
            for _ in 0..200{
                if let Some(combat)=run.state.combat.as_ref(){assert_eq!(combat.maintains_public_knowledge(),maintained);break}
                let d=run.decision().unwrap();let action=d.candidates[0].action.clone();run.step(&action).unwrap();
            }
            assert!(run.state.combat.is_some(),"reached a combat");
        }
    }

    #[test]
    fn combat_scope_rejects_impossible_starts(){
        let rng=||RunRng::new([3;32],Partition::Test,0);
        let mut s=start(30,vec![]);s.ascension=21;
        assert_eq!(Run::prepare_standard_combat(s,rng()).err(),Some(RunError::InvalidAscension(21)));
        let mut s=start(30,vec![]);s.burning=Some(BurningEliteBuff::Strength);
        assert!(Run::prepare_standard_combat(s.clone(),rng()).is_err(),"burning on a monster");
        s.encounter=EncounterId::GremlinNob;
        let mut run=Run::prepare_standard_combat(s,rng()).unwrap();run.enter_standard_combat().unwrap();
        assert!(run.combat_identity().unwrap().burning);
        // Ordinary Runs never enter the combat scope.
        let mut ordinary=Run::new(RunConfig{character:Character::Ironclad,ascension:0},CreationProfile::experienced(),rng()).unwrap();
        assert_eq!(ordinary.enter_standard_combat().err(),Some(RunError::InvalidAction));
        assert!(ordinary.combat_scope_terminal().is_none());
    }
}
