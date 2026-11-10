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

struct OwnedStreamIdentity{act:u8,encounter:&'static str,visit:u64}
impl OwnedStreamIdentity{fn as_ref(&self)->crate::combat::belief::StreamIdentity<'static>{crate::combat::belief::StreamIdentity{act:self.act,encounter:self.encounter,visit:self.visit}}}

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
    /// M34s-b b1 (design §4.6.3): the combat-scope transplant of the live combat — one positive list of the Run-side
    /// inputs the combat and its Standard view read (character, ascension, act, floor, room label, HP, gold, deck and
    /// permanent identity counter, keys, combat progress and identity, item admission and custody) plus an exact copy of
    /// the combat, in the settlement-free mode. Run-level hidden and progression state (Run RNG, map, room loop, pools,
    /// events, history, ledgers, Neow, selections) is absent by construction; the Run RNG is a fixed zero root, which
    /// no live combat action draws. No canonicalisation: `combat/belief.rs` canonicalises this copy.
    pub fn combat_scope(&self)->Result<Run,RunError>{
        if let Some(fault)=&self.state.fault{return Err(RunError::Faulted(fault.clone()))}
        if self.state.terminal.is_some(){return Err(RunError::Terminal)}
        let (Some(combat),Some(active))=(self.state.combat.as_ref(),self.state.active_combat.as_ref()) else {return Err(RunError::InvalidAction)};
        let s=&self.state;
        let profile=CreationProfile::experienced();
        let state=RunState{
            room_loop:None,reward_ledger:Default::default(),authored_combat:false,training_encounter:None,last_combat_checkpoint:None,
            combat_progress:s.combat_progress,active_combat:Some(active.clone()),last_combat:None,
            character:s.character,ascension:s.ascension,hp:s.hp,max_hp:s.max_hp,gold:s.gold,
            deck:s.deck.clone(),relics:s.relics.clone(),potions:s.potions.clone(),keys:s.keys,act:s.act,floor:s.floor,room:s.room,
            act4_enabled:true,neow_eligible:false,seen_bosses:profile.seen_bosses,stored_note:profile.stored_note,terminal:None,map:None,
            history:RunHistory::default(),relic_counters:s.relic_counters.clone(),combat:Some(combat.clone()),selected_neow:None,fault:None,
            last_selection:None,last_resumed_parent:None,rng:RunRng::new([0;32],crate::combat::rng::Partition::Test,0),pending:None,
            next_instance:s.next_instance,resume_count:0,card_admission:s.card_admission,item_admission:s.item_admission,relic_pools:None,
            relic_cards:s.relic_cards.clone(),potion_offset:0,card_rarity_offset:None,event_pools:None,potion_potencies:s.potion_potencies.clone(),
            we_meet_again:s.we_meet_again,combat_scope:Some(CombatScope{encounter:active.encounter,burning:None}),
        };
        Ok(Run{state,candidates:vec![],next_choice:0})
    }
    /// M34s-b (design §4.6.3): the canonical public root of the live combat — the combat-scope transplant with the
    /// combat canonicalised (`combat/belief.rs`). Errors name a class of reason, never a value.
    pub fn public_root_run(&self)->Result<Run,crate::combat::belief::Unsupported>{
        use crate::combat::belief::Unsupported;
        if self.state.fault.is_some(){return Err(Unsupported::Faulted)}
        let mut root=self.combat_scope().map_err(|e|match e{RunError::Terminal=>Unsupported::Terminal,_=>Unsupported::NotAtDecision})?;
        let id=root.stream_identity().ok_or(Unsupported::NotAtDecision)?;
        let combat=root.state.combat.as_ref().unwrap().belief_canonical(id.as_ref())?;
        root.state.combat=Some(combat);
        Ok(root)
    }
    /// M34s-b (design §4.6.2): the world of sample `key` from this canonical root (`public_root_run`); its Run RNG root is
    /// research-derived as well, so nothing in the world derives from a gameplay key.
    pub fn belief_sample(&self,key:&[u8;32])->Result<Run,crate::combat::belief::Unsupported>{
        use crate::combat::belief::Unsupported;
        if self.state.combat_scope.is_none(){return Err(Unsupported::NotAtDecision)}
        let id=self.stream_identity().ok_or(Unsupported::NotAtDecision)?;
        let combat=self.state.combat.as_ref().ok_or(Unsupported::NotAtDecision)?.belief_sample(key,id.as_ref())?;
        let mut world=self.clone();
        world.state.rng=RunRng::new(crate::combat::rng::hmac_sha256(key,b"r2-run-rng"),crate::combat::rng::Partition::Test,0);
        world.state.combat=Some(combat);
        Ok(world)
    }
    /// The public knowledge of the live combat in normal form, as digest input bytes (design §4.6.3): the draw
    /// knowledge and every actor's pre-roll candidates. `None` without supported knowledge.
    pub fn public_knowledge_bytes(&self)->Option<Vec<u8>>{
        let (draw,prerolls)=self.state.combat.as_ref()?.belief_support().ok()?;
        Some(format!("{draw:?}\0{prerolls:?}").into_bytes())
    }
    /// Final review I1: whether `action` picks a draw-pile copy chosen by hidden instance order
    /// (`Combat::pick_depends_on_hidden_order`); public.
    pub fn pick_depends_on_hidden_order(&self,action:&GameAction)->bool{
        let GameAction::Combat(crate::combat::CombatAction::SelectCard{instance})=action else{return false};
        self.state.combat.as_ref().is_some_and(|c|c.pick_depends_on_hidden_order(*instance))
    }
    /// The heap the live combat's draw knowledge retains (public: a function of the knowledge; final review I2 measurement).
    pub fn public_knowledge_heap_bytes(&self)->Option<usize>{self.state.combat.as_ref()?.draw_knowledge().ok().map(|k|k.heap_bytes())}
    fn stream_identity(&self)->Option<OwnedStreamIdentity>{
        let active=self.state.active_combat.as_ref()?;
        Some(OwnedStreamIdentity{act:self.state.act,encounter:active.encounter.as_str(),visit:active.visit})
    }
    /// Fuzz oracle only (never public): the live combat, for the a6 draw-knowledge support check.
    #[cfg(any(test, feature = "fuzz-diagnostics"))]
    pub fn fuzz_combat(&self)->Option<&crate::combat::Combat>{self.state.combat.as_ref()}
    /// Fuzz oracle only (never public): the live combat, mutable, for the M34s-b P2 perturbation check.
    #[cfg(any(test, feature = "fuzz-diagnostics"))]
    pub fn fuzz_combat_mut(&mut self)->Option<&mut crate::combat::Combat>{self.state.combat.as_mut()}
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

    /// M34s-b b1 (design §4.6.3): the transplant is a positive list of combat inputs; Run-level hidden and progression
    /// state (Run RNG, map, room loop, pools, events, history, ledgers) is absent, so perturbing it changes nothing.
    #[test]
    fn transplant_holds_no_run_level_state(){
        let mut run=Run::new_room_scenario(RunConfig{character:Character::Silent,ascension:20},
            CreationProfile{neow_eligible:false,..CreationProfile::experienced()},RunRng::new([9;32],Partition::Test,0),super::rooms::RoomScenario::Standard).unwrap();
        for _ in 0..200{ if run.state.combat.is_some(){break} let a=run.decision().unwrap().candidates[0].action.clone();run.step(&a).unwrap(); }
        assert!(run.state.combat.is_some(),"reached a combat");
        let mut other=run.clone();
        other.state.rng=RunRng::new([77;32],Partition::Final,9);other.state.map=None;other.state.room_loop=None;
        other.state.history.rooms.clear();other.state.seen_bosses=Default::default();other.state.relic_pools=None;other.state.event_pools=None;
        other.state.card_rarity_offset=Some(-40);other.state.potion_offset=30;other.state.reward_ledger=Default::default();
        other.state.last_combat=None;other.next_choice=12_345;other.state.stored_note=(CardId::Strike,3);other.state.neow_eligible=true;
        let (a,b)=(run.combat_scope().unwrap(),other.combat_scope().unwrap());
        assert!(a==b,"Run-level state never enters the transplant");
        let s=&a.state;
        assert!(s.map.is_none()&&s.room_loop.is_none()&&s.relic_pools.is_none()&&s.event_pools.is_none()&&s.history==RunHistory::default());
        assert_eq!((s.rng.clone(),s.combat_scope.is_some(),a.next_choice),(RunRng::new([0;32],Partition::Test,0),true,0));
        assert!(s.combat.as_ref().unwrap().maintains_public_knowledge());
        // Outside a live combat, after a fault or at a Run terminal there is no transplant.
        let mut idle=Run::new(RunConfig{character:Character::Ironclad,ascension:0},CreationProfile::experienced(),RunRng::new([1;32],Partition::Test,0)).unwrap();
        assert_eq!(idle.combat_scope().err(),Some(RunError::InvalidAction));
        idle.install_terminal_for_test(RunTerminal::Death);
        assert_eq!(idle.combat_scope().err(),Some(RunError::Terminal));
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
