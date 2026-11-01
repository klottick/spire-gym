//! M26 complete card capability (plan Task M1): the overlay is inert for a
//! character's own cards (design §5.2, §6.4; seam map H0 §6 scripts).
use super::super::*;
use crate::{content::{Catalog,CardId,Character,EncounterId},items::CombatItemContext};
use CardId::*;

fn own_cards(character:Character)->[CardId;5]{
    match character{
        Character::Ironclad=>[Flex,PommelStrike,Inflame,Armaments,ShrugItOff],
        Character::Silent=>[Backflip,DeadlyPoison,Footwork,Acrobatics,BladeDance],
        Character::Defect=>[BallLightning,Coolheaded,Defragment,ChargeBattery,Loop],
        Character::Watcher=>[FlurryOfBlows,Worship,Protect,Foresight,MentalFortress],
    }
}
fn base_context(character:Character)->CombatItemContext{
    let relic=Catalog::standard().starter_relic(character).unwrap();
    let items=CombatItemContext::new(vec![relic],vec![None;3]);
    match character{
        Character::Ironclad=>items,
        Character::Silent=>items.with_m15_profile(),
        Character::Defect=>items.with_m20_profile(),
        Character::Watcher=>items.with_m25_profile(),
    }
}
/// The unstripped Act 1 Cultist entry (seed `[41;32]`, as `m8_shapes::entry`).
fn cultist_entry(character:Character)->CombatEntry{
    enemies::setup(EncounterId::Cultist,1,0,
        &mut crate::run::RunRng::new([41;32],rng::Partition::Test,0),
        enemies::SetupContext{
            player:PlayerState{hp:80,max_hp:80,block:0,energy:3,max_energy:3,draw_per_turn:5,powers:vec![]},
            player_powers:vec![],character,
            cards:vec![permanent_live_card(CardId::Defend,CardInstanceId(1),0)],
            opening_draw:1,encounter_visit:0,
        }).unwrap()
}
fn own_colour_combat(character:Character,complete:bool)->Combat{
    let mut e=cultist_entry(character);
    let deck=Catalog::standard().starter_deck(character).unwrap().iter().copied().chain(own_cards(character));
    e.cards=deck.enumerate().map(|(i,id)|{
        let instance=CardInstanceId(i as u64+1);
        if character==Character::Watcher{m25_watcher_live_card(id,instance,0).unwrap()}else{permanent_live_card(id,instance,0)}
    }).collect();
    e.opening_draw=5;
    let items=base_context(character);
    let mut items=Some(if complete{items.with_complete_cards()}else{items});
    Combat::enter_with_item_context(e,&mut items).unwrap()
}
/// "First legal non-end-turn action, else End Turn" to the combat end (bound 400);
/// an open selection is confirmed as soon as that is legal and never deselected.
fn script(mut c:Combat)->Vec<String>{
    let mut log=vec![];
    for _ in 0..400{
        log.push(format!("{:?}",c.public_full()));
        let Ok(actions)=c.legal_actions_full() else {break};
        log.push(format!("{actions:?}"));
        let Some(action)=actions.iter().find(|a|matches!(a,CombatAction::ConfirmSelection))
            .or_else(||actions.iter().find(|a|!matches!(a,CombatAction::EndTurn|CombatAction::DeselectCard{..})))
            .or(actions.first()).cloned() else {break};
        c.step_full(&action).unwrap_or_else(|e|panic!("{action:?}: {e:?}"));
    }
    log.push(format!("{:?}",c.exit()));
    log
}

#[test]
fn m26_capability_overlay_inert(){
    for character in [Character::Ironclad,Character::Silent,Character::Defect,Character::Watcher]{
        let base=script(own_colour_combat(character,false));
        let mut overlay=script(own_colour_combat(character,true));
        assert!(base.last().unwrap().starts_with("Ok("),"{character:?} script reaches the combat end in {} entries: {}\n{}",
            base.len(),base.last().unwrap(),base[base.len().saturating_sub(4)..].join("\n"));
        assert_eq!(base.len(),overlay.len(),"{character:?} step count");
        // The one intended difference: the exit's item context prints the overlay flag.
        let exit=overlay.last_mut().unwrap();
        assert_eq!(exit.matches(", complete_cards: true").count(),1,"{character:?} exit carries the flag once");
        *exit=exit.replace(", complete_cards: true","");
        for (step,(b,o)) in base.iter().zip(&overlay).enumerate(){
            let at=b.bytes().zip(o.bytes()).position(|(x,y)|x!=y).unwrap_or(b.len().min(o.len()));
            assert!(b==o,"{character:?} public bytes diverge at log entry {step}, byte {at}:\n base…{}\n over…{}",
                &b[at.saturating_sub(120)..(at+200).min(b.len())],&o[at.saturating_sub(120)..(at+200).min(o.len())]);
        }
    }
}

/// Finite profile lists with their accepted lengths; M26 widens none of them.
fn finite_lists()->Vec<(&'static str,Vec<CardId>,usize)>{
    use crate::content::CardRarity::*;
    let red=[Basic,Common,Uncommon,Rare].into_iter().flat_map(|r|crate::run::rooms::m10_content::red_candidates(r).iter().copied()).collect();
    vec![
        ("M10 red",red,75),
        ("M15 Silent Heart",crate::run::M15_SILENT_HEART_CARDS.to_vec(),0),
        ("M16 Defect",crate::content::defect::M16_DEFECT_CARDS.to_vec(),0),
        ("M19 Defect Act3",crate::run::M19_DEFECT_ACT3_CARDS.to_vec(),0),
        ("M20 Defect",crate::content::defect::M20_DEFECT_CARDS.to_vec(),0),
        ("M20 Defect Heart",crate::run::M20_DEFECT_HEART_CARDS.to_vec(),0),
        ("M21 Watcher",crate::content::watcher::M21_WATCHER_CARDS.to_vec(),28),
        ("M24 Watcher Act3",crate::run::M24_WATCHER_ACT3_CARDS.to_vec(),0),
        ("M25 Watcher",crate::content::watcher::M25_WATCHER_CARDS.to_vec(),75),
        ("M25 Watcher Heart",crate::run::M25_WATCHER_HEART_CARDS.to_vec(),76),
        ("training",crate::run::TRAINING_CARDS.to_vec(),0),
    ]
}

#[test]
fn m26_identity_complete_unions(){
    use crate::content::complete::*;
    let rows=Catalog::standard().cards();
    let permanent=rows.iter().filter(|row|supports_permanent(row.id)).count();
    let generated_only=rows.iter().filter(|row|supports_generated(row.id)&&!supports_permanent(row.id)).map(|row|row.id).collect::<Vec<_>>();
    let choice=rows.iter().filter(|row|is_choice_only(row.id)).map(|row|row.id).collect::<Vec<_>>();
    assert_eq!(permanent,352,"complete permanent union");
    let mut expected_generated=vec![Shiv,Miracle,Smite,Safety,Insight,ThroughViolence,Beta,Omega,Expunger,Burn,Dazed,Slimed,Wound,Void];
    let mut actual=generated_only.clone();actual.sort_by_key(|id|id.as_str());expected_generated.sort_by_key(|id|id.as_str());
    assert_eq!(actual,expected_generated,"generated-only: 9 children + 5 statuses");
    assert_eq!(choice,vec![FameAndFortune,BecomeAlmighty,LiveForever].into_iter().filter(|id|choice.contains(id)).collect::<Vec<_>>());
    assert_eq!(choice.len(),3,"choice-only: Wish's options");
    assert!(choice.iter().all(|id|!supports_generated(*id)&&!supports_permanent(*id)));
    let nonremovable=rows.iter().filter(|row|supports_permanent(row.id)&&!is_removable(row.id)).map(|row|row.id).collect::<Vec<_>>();
    let mut nonremovable_sorted=nonremovable.clone();nonremovable_sorted.sort_by_key(|id|id.as_str());
    assert_eq!(nonremovable_sorted,vec![AscendersBane,CurseOfTheBell,Necronomicurse]);
    for (name,list,len) in finite_lists(){
        if len!=0{assert_eq!(list.len(),len,"{name} unchanged");}
        for id in list{assert!(supports_permanent(id),"{name} member {id:?} is in the complete permanent union");}
    }
}

const CHARACTERS:[Character;4]=[Character::Ironclad,Character::Silent,Character::Defect,Character::Watcher];
fn heart_context(character:Character)->CombatItemContext{
    let items=CombatItemContext::new(vec![],vec![None;3]);
    match character{Character::Ironclad=>items,Character::Silent=>items.with_m15_profile(),Character::Defect=>items.with_m20_profile(),Character::Watcher=>items.with_m25_profile()}
}
fn enter_one(character:Character,card:LiveCard,complete:bool)->Result<Combat,CombatEntryFailure>{
    let mut e=cultist_entry(character);e.cards=vec![card];e.opening_draw=1;
    let items=heart_context(character);
    let mut items=Some(if complete{items.with_complete_cards()}else{items});
    Combat::enter_with_item_context(e,&mut items)
}
fn upgrade_counts(id:CardId)->Vec<u32>{
    if crate::run::complete_upgrades_valid(id,1){vec![0,1]}else{vec![0]}
}

#[test]
fn m26_capability_combat_admission(){
    use crate::content::complete::*;
    let rows=Catalog::standard().cards();
    for character in CHARACTERS{
        for row in rows.iter().filter(|row|supports_generated(row.id)){
            for up in upgrade_counts(row.id){
                let card=complete_live_card(row.id,CardInstanceId(1),up).unwrap_or_else(|e|panic!("{:?}+{up}: {e:?}",row.id));
                let combat=enter_one(character,card.clone(),true).unwrap_or_else(|e|panic!("{character:?} {:?}+{up}: {e:?}",row.id));
                let live=combat.state.piles.all(combat.state.in_play.as_ref()).find_map(|owned|owned.full().filter(|c|c.instance==CardInstanceId(1)))
                    .unwrap_or_else(||panic!("{character:?} {:?} physical",row.id));
                assert_eq!((live.id,live.upgrades),(row.id,up));
            }
        }
        for id in [FameAndFortune,BecomeAlmighty,LiveForever]{
            assert!(complete_live_card(id,CardInstanceId(1),0).is_err(),"{id:?} never constructs");
        }
    }
    // A purple card under an Ironclad context carries the Watcher owner's fields.
    for id in [Eruption,Wallop,Blasphemy,Protect]{
        for up in [0,1]{
            let watcher=m25_watcher_live_card(id,CardInstanceId(1),up).unwrap();
            let complete=complete_live_card(id,CardInstanceId(1),up).unwrap();
            assert_eq!(complete,watcher,"{id:?}+{up} Watcher fields");
            let entered=enter_one(Character::Ironclad,complete,true).unwrap();
            let live=entered.state.piles.hand.iter().find_map(|o|o.full().cloned()).unwrap();
            assert_eq!(live.local,watcher.local,"{id:?}+{up} Watcher fields after Ironclad entry");
        }
    }
    // Without the flag, the finite rejections stay.
    assert!(enter_one(Character::Watcher,permanent_live_card(Strike,CardInstanceId(1),0),false).is_err(),"M25 rejects a red permanent");
    assert!(enter_one(Character::Watcher,permanent_live_card(Strike,CardInstanceId(1),0),true).is_ok());
    let plain=format!("{:?}",heart_context(Character::Ironclad));
    assert!(!plain.contains("complete_cards"),"Debug unchanged without the flag");
    assert!(format!("{:?}",heart_context(Character::Ironclad).with_complete_cards()).ends_with(", complete_cards: true }"));
}

#[test]
fn m26_capability_misc_growth_journal(){
    for (id,start,amount) in [(GeneticAlgorithm,1,3),(RitualDagger,15,5)]{
        let card=complete_live_card(id,CardInstanceId(1),0).unwrap();
        assert_eq!(card.misc,start,"{id:?} obtain misc");
        let mut c=enter_one(Character::Ironclad,card,true).unwrap();
        cards::defect::record_misc_growth(&mut c.state,CardInstanceId(1),amount).unwrap();
        let live=c.state.piles.all(c.state.in_play.as_ref()).find_map(|o|o.full().filter(|l|l.instance==CardInstanceId(1)).cloned()).unwrap();
        assert_eq!(live.misc,start+amount);
        assert_eq!(c.state.defect.permanent_mutations,vec![PermanentCardMutation{ordinal:1,instance:CardInstanceId(1),id,misc:start+amount}]);
    }
    let mut c=enter_one(Character::Ironclad,complete_live_card(Strike,CardInstanceId(1),0).unwrap(),true).unwrap();
    assert!(cards::defect::record_misc_growth(&mut c.state,CardInstanceId(1),3).is_err(),"only misc-carrying families grow");
}
