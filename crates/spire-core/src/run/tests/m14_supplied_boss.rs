//! Explicitly supplied test evidence; it is never canonical normal progression.
use super::super::*;
use crate::content::{CardId, EncounterId, PotionId, RelicId};

#[path = "../../../examples/m14_support/mod.rs"]
#[allow(dead_code)]
mod frozen;

mod fixture;
mod completion;
mod fixture_tests;

pub(super) const CASES: [(u64, frozen::Scenario); 6] = [
    frozen::FIXED_CASES[0], frozen::FIXED_CASES[1], frozen::FIXED_CASES[2],
    frozen::FIXED_CASES[4], frozen::FIXED_CASES[5], frozen::FIXED_CASES[3],
];
pub(super) const DECK: [CardId; 14] = [
    CardId::NoxiousFumes, CardId::NoxiousFumes, CardId::Catalyst, CardId::Catalyst,
    CardId::Backflip, CardId::Backflip, CardId::Backflip,
    CardId::Deflect, CardId::Deflect, CardId::Deflect,
    CardId::DeadlyPoison, CardId::DeadlyPoison,
    CardId::CloakAndDagger, CardId::WellLaidPlans,
];
pub(super) const ACQUIRE_RELICS: [RelicId; 11] = [
    RelicId::Pear, RelicId::Mango, RelicId::Strawberry, RelicId::SlaversCollar,
    RelicId::SneckoSkull, RelicId::OddlySmoothStone, RelicId::BagOfPreparation,
    RelicId::Lantern, RelicId::Anchor, RelicId::BronzeScales, RelicId::RedMask,
];
pub(super) const POTIONS: [Option<PotionId>; 3] = [Some(PotionId::DexterityPotion); 3];

fn full_native_start(initial: &PublicDecision, target: EncounterId) -> bool {
    use crate::content::EnemyId;
    let Some(view)=initial.combat.as_ref() else { return false; };
    if initial.content_profile!=Some(RoomScenario::M14SilentAct3)
        || (initial.act,initial.floor)!=(3,50) || initial.terminal.is_some()
        || view.player.hp!=101 || view.player.max_hp!=101
        || view.enemies.iter().any(|enemy|enemy.hp<=0 || enemy.hp!=enemy.max_hp) { return false; }
    let has=|id,hp|view.enemies.iter().any(|enemy|enemy.enemy_id==id && enemy.hp==hp);
    match target {
        EncounterId::AwakenedOne=>view.enemies.len()==3 && has(EnemyId::AwakenedOne,300)
            && view.enemies.iter().filter(|enemy|enemy.enemy_id==EnemyId::Cultist).count()==2,
        EncounterId::TimeEater=>view.enemies.len()==1 && has(EnemyId::TimeEater,456),
        EncounterId::DonuAndDeca=>view.enemies.len()==2 && has(EnemyId::Donu,250) && has(EnemyId::Deca,250),
        _=>false,
    }
}

fn proof_from_attempt(
    initial: &PublicDecision, entry: &RunTransition, target: EncounterId,
    caller: &frozen::Caller, result: &Result<frozen::CallerOutcome,String>,
) -> completion::Proof {
    let entries=entry.committed_events.iter().filter_map(|event|match event {
        CommittedEvent::CombatEntered{visit,encounter}=>Some((*visit,*encounter)), _=>None,
    }).collect::<Vec<_>>();
    let mut proof=completion::Proof {
        full_hp_start: full_native_start(initial,target)
            && entries.len()==1 && entry.committed_events.iter().any(|event|matches!(event,CommittedEvent::FloorEntered{act:3,floor:50}))
            && !entry.committed_events.iter().any(|event|matches!(event,CommittedEvent::CombatFinished(_))),
        entry: entries.first().copied(), target, victories:vec![], proceed:vec![], dialogs:vec![], ending:vec![],
        endpoint:caller.final_decision.as_ref().and_then(|decision|decision.terminal.map(|terminal|(decision.act,decision.floor,terminal))),
        result_no_keys:matches!(result,Ok(frozen::CallerOutcome::Act3NoKeys)), forbidden_advance:false, ending_reward:0,
    };
    for (index,step) in caller.trace.iter().enumerate() {
        proof.forbidden_advance |= step.decision.act>3 || step.decision.floor>50;
        if let GameAction::Choose(id)=step.action {
            if let Some(room)=&step.decision.room {
                if let Some(row)=room.choices.iter().find(|row|row.id==id && row.status==RoomChoiceStatus::Offered) {
                    match (&room.kind,&row.kind) {
                        (RoomViewKind::Act3BossComplete,RoomChoiceKind::Proceed)=>{
                            proof.proceed.push(index); proof.ending_reward+=step.transition.reward;
                        },
                        (RoomViewKind::Act3Ending,RoomChoiceKind::Act3Continue{stage})=>{
                            proof.dialogs.push((index,*stage)); proof.ending_reward+=step.transition.reward;
                        }, _=>{},
                    }
                }
            }
        }
        for event in &step.transition.committed_events {
            match event {
                CommittedEvent::CombatFinished(record)=>proof.victories.push((index,record.clone())),
                CommittedEvent::Ended(RunTerminal::Act3NoKeys)=>proof.ending.push(index),
                CommittedEvent::FloorEntered{floor,..} if *floor>50=>proof.forbidden_advance=true,
                CommittedEvent::ActAdvanced{..}|CommittedEvent::Ended(RunTerminal::HeartVictory)=>proof.forbidden_advance=true,
                _=>{},
            }
        }
    }
    proof
}

// This test is the entire six-case supplied evaluation. It never changes any
// Run state after prepare's real Move; only the frozen public caller steps it.
#[test]
fn six_full_boss_to_nokeys() {
    use std::io::Write;
    let directory=std::env::var_os("SPIRE_M14_SUPPLIED_TRACE_DIR").map(std::path::PathBuf::from);
    let mut errors=Vec::new();
    let mut qualifying=0;
    let mut completed=0;
    for (index,(seed,scenario)) in CASES.into_iter().enumerate() {
        let mut caller=frozen::Caller::new_m14(8192).unwrap();
        let prepared=fixture::prepare(seed,scenario);
        let (result,details,proof)=match prepared {
            Ok(mut prepared)=>{
                let initial=prepared.run.decision();
                let initial_map=prepared.run.public_map();
                let resources=format!("owned_deck={:#?}\nnext_instance={}\nact4_enabled={}\ninitial_public={initial:#?}\ninitial_map={initial_map:#?}\nsetup_entry={:#?}",
                    prepared.run.state.deck,prepared.run.state.next_instance,prepared.run.state.act4_enabled,prepared.entry);
                let result=frozen::rollout_m14(&mut prepared.run,&mut caller,false);
                let proof=match (&initial,&prepared.entry) {
                    (Ok(initial),Some(entry))=>Some(proof_from_attempt(initial,entry,scenario.act3_boss,&caller,&result)),
                    _=>None,
                };
                let endpoint=prepared.run.decision();
                (result,format!("{resources}\nactual_endpoint={endpoint:#?}"),proof)
            },
            Err(error)=>(Err(format!("supplied preparation: {error}")),"no Run was prepared".into(),None),
        };
        // Every Result and full public trace is saved before derived assertions.
        if let Some(directory)=&directory {
            let path=directory.join(format!("{:02}-seed-{seed}-{:?}-{:?}-{:?}.trace.txt",index+1,scenario.act1_boss,scenario.act2_boss,scenario.act3_boss));
            let retained=(||->std::io::Result<()> {
                let mut file=std::fs::OpenOptions::new().write(true).create_new(true).open(path)?;
                writeln!(file,"evidence_kind=supplied-full-boss\ncaller_policy={}\nseed={seed}\nscenario={scenario:?}\nlimit=8192\nresult={result:?}\naccepted_actions={}\n{details}\nproof={proof:#?}\n{caller:#?}",frozen::CALLER_POLICY_REVISION,caller.accepted())
            })();
            if let Err(error)=retained { errors.push(format!("case {index} retention: {error}")); }
        }
        let won=proof.as_ref().is_some_and(completion::qualifies);
        qualifying+=usize::from(won); completed+=1;
        match &result {
            Ok(frozen::CallerOutcome::Death|frozen::CallerOutcome::EmptyDeck)=>{},
            Ok(frozen::CallerOutcome::Act3NoKeys) if won=>{},
            _=>errors.push(format!("case {index}: invalid supplied outcome/proof {result:?}")),
        }
        eprintln!("M14 SUPPLIED case={} seed={seed} scenario={scenario:?} result={result:?} accepted={} qualifying={won} endpoint={:?}",index+1,caller.accepted(),caller.final_decision.as_ref().map(|decision|(decision.act,decision.floor,decision.terminal)));
    }
    eprintln!("M14 SUPPLIED completed={completed} qualifying_full_boss_NoKeys={qualifying} errors={errors:?}; canonical_normal_completion=UNPROVEN");
    assert_eq!(completed,6);
    assert!(errors.is_empty(),"supplied operational/proof/retention failures: {errors:?}");
    assert!(qualifying>0,"no supplied full-HP boss Victory/Proceed/dialog/NoKeys among all six");
}
