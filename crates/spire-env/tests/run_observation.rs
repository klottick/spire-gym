use spire_env::*;
use spire_core::{combat::{self,CombatAction,CombatCardRef},content::{CardId,PotionId},run::*};
#[path="../../spire-core/examples/m9_support/mod.rs"] mod support;

fn initial()->Run{support::start(11,support::M9BossHistory::TimeEaterUnseen).unwrap()}
fn combat_run()->Run{
    let mut r=initial();let first=r.decision().unwrap().candidates[0].action.clone();r.step(&first).unwrap();
    for _ in 0..16{let d=r.decision().unwrap();if d.combat.is_some(){return r}
        let action=d.candidates.iter().find(|c|matches!(c.action,GameAction::Move(_))).unwrap().action.clone();r.step(&action).unwrap();
    }panic!("no combat")
}
fn g<'a>(o:&'a RunObservation,key:&str)->&'a RunFeatureGroup{o.group(key).unwrap()}

#[test]
fn run_schema_spaces_are_fixed_and_separate_from_combat(){
    let caps=RunCapacities::m9_default();let spec=run_observation_spec(&caps).unwrap();let d=spec.descriptor();
    assert_eq!(d["schema_id"],RUN_OBSERVATION_SCHEMA_ID);assert_ne!(d["schema_id"],observation_spec().descriptor()["schema_id"]);
    assert_eq!(d["action_space"]["n"],2048);assert_eq!(d["capacities"]["combat_cards"],1024);
    let context=d["tensors"].as_array().unwrap().iter().find(|t|t["key"]=="context").unwrap();
    assert_eq!(context["dtype"],"int64");assert_eq!(context["shape"],serde_json::json!([18]));assert_eq!(context["fields"][0],"character");
    assert_eq!(d["vocabularies"]["relic"].as_array().unwrap().iter().filter(|x|**x=="bronze_scales").count(),1);
    for table in d["vocabularies"].as_object().unwrap().values(){let rows=table.as_array().unwrap();for (i,name) in rows.iter().enumerate(){assert!(!rows[..i].contains(name),"duplicate vocabulary name {name}");}}
    let power=d["tensors"].as_array().unwrap().iter().find(|t|t["key"]=="powers").unwrap();assert_eq!(power["columns"][3]["low"],i32::MIN);assert_eq!(power["columns"][2]["vocabulary"],"power");
    let mut larger=caps.clone();larger.actions+=1;assert_ne!(spec.fingerprint(),run_observation_spec(&larger).unwrap().fingerprint());
    larger.actions=usize::MAX;assert!(run_observation_spec(&larger).is_err());
    let o=encode_run(&initial().decision().unwrap(),None,&caps).unwrap();
    for tensor in d["tensors"].as_array().unwrap(){
        let key=tensor["key"].as_str().unwrap();assert!(key=="action_mask"||o.groups.iter().any(|g|key==g.key||key==format!("{}_known",g.key)||key==format!("{}_occupied",g.key)));
    }
}

#[test]
fn run_encoding_covers_room_combat_selection_and_potion_actions(){
    let caps=RunCapacities::m9_default();let r=combat_run();let mut d=r.decision().unwrap();
    let card=d.combat.as_ref().unwrap().hand[0].clone();
    d.combat.as_mut().unwrap().player.powers.push(combat::FullPowerView{id:combat::FullPowerId("strength"),amount:-3});
    d.combat.as_mut().unwrap().enemies[0].intent_damage=None;d.combat.as_mut().unwrap().enemies[0].half_dead=true;
    d.combat.as_mut().unwrap().selection=Some(combat::SelectionView{candidates:vec![card.clone()],required:1,selected:vec![],disposition:combat::SelectionDisposition::Exhaust});
    d.candidates=vec![Candidate{id:ChoiceId(999),action:GameAction::Combat(CombatAction::SelectCard{instance:card.instance.0}),label:"select",object:None,neow_offer:None}];
    let o=encode_run(&d,r.public_map().as_ref(),&caps).unwrap();
    assert_eq!(o.action_mask.iter().filter(|x|**x==1).count(),1);assert_eq!(g(&o,"powers").values[3],-3);
    assert_eq!(g(&o,"enemies").known[5],0);assert_eq!(g(&o,"enemies").values[8],1);assert_eq!(g(&o,"selection_counts").values,[1,0]);
    assert_eq!(g(&o,"selection").values[2],1);assert_eq!(g(&o,"selection").known[3],1);
    assert_eq!(g(&o,"actions").values[2],0); // selection-only action references visible combat row0
    let saved=o.clone();let mut again=encode_run(&d,r.public_map().as_ref(),&caps).unwrap();again.groups[0].values[0]=999;assert_eq!(o,saved);

    let mut d=initial().decision().unwrap();let object=d.objects[0].clone();
    d.selection=Some(PublicSelection{required:2,selected_count:1,eligible:vec![object.clone()]});
    d.candidates=vec![Candidate{id:ChoiceId(4),action:GameAction::Select(SelectionId(555)),label:"remove",object:Some(object),neow_offer:None}];
    d.room=Some(RoomView{kind:RoomViewKind::BossRelicChild,choices:vec![]});
    let o=encode_run(&d,None,&caps).unwrap();assert_eq!(g(&o,"selection").known[3],0);assert_eq!(g(&o,"selection_counts").values,[2,1]);
    d.selection=None;d.player.potions=vec![Some(PotionId::FirePotion);3];d.candidates=vec![Candidate{id:ChoiceId(5),action:GameAction::DiscardPotion{slot:2},label:"discard",object:None,neow_offer:None}];
    let o=encode_run(&d,None,&caps).unwrap();assert_eq!(g(&o,"potions").occupied.as_ref().unwrap().iter().filter(|v|**v==1).count(),3);assert_eq!(g(&o,"actions").values[2],2);
}

#[test]
fn run_encoding_hides_private_ids_future_data_and_unknown_lengths(){
    let r=combat_run();let d=r.decision().unwrap();let map=r.public_map();let caps=RunCapacities::m9_default();let o=encode_run(&d,map.as_ref(),&caps).unwrap();
    let mut changed=d.clone();let shift=1_000_000;
    for card in &mut changed.objects{card.instance+=shift;}
    let c=changed.combat.as_mut().unwrap();
    for card in c.hand.iter_mut().chain(&mut c.discard).chain(&mut c.exhaust).chain(&mut c.played_powers){card.instance.0+=shift;if let CombatCardRef::Permanent(id)=&mut card.origin{id.0+=shift;}}
    for candidate in &mut changed.candidates{candidate.id.0+=shift;match &mut candidate.action{GameAction::Combat(CombatAction::PlayCard{instance,..}|CombatAction::SelectCard{instance})=>*instance+=shift,_=>{}}}
    assert_eq!(encode_run(&changed,map.as_ref(),&caps).unwrap(),o);
    assert_eq!(g(&o,"combat_cards").occupied.as_ref().unwrap().iter().filter(|v|**v==1).count(),d.combat.as_ref().unwrap().hand.len());
    assert!(g(&o,"orbs").known.iter().all(|v|*v==0));
    assert!(g(&o,"map_nodes").known.chunks(8).all(|row|row[4]==0)); // no hidden question outcomes
    let mut unknown=d.clone();unknown.combat.as_mut().unwrap().hand[0].id=CardId::Unknown("strike");
    assert!(matches!(encode_run(&unknown,map.as_ref(),&caps),Err(RunSessionError::Input(_)))); // never alias Unknown to a known key
}

#[test]
fn run_encoding_capacity_errors_keep_owned_frontiers(){
    let r=initial();let d=r.decision().unwrap();let caps=RunCapacities::m9_default();let saved=d.clone();
    let mut small=caps.clone();small.permanent_cards=9;
    assert_eq!(encode_run(&d,None,&small),Err(RunSessionError::Capacity{group:"permanent_cards",required:10,capacity:9}));assert_eq!(d,saved);
    small=caps.clone();small.actions=1;assert_eq!(encode_run(&d,None,&small),Err(RunSessionError::Capacity{group:"actions",required:2,capacity:1}));
    small=caps.clone();small.offers=1;assert_eq!(encode_run(&d,None,&small),Err(RunSessionError::Capacity{group:"offers",required:2,capacity:1}));
    let r=combat_run();let d=r.decision().unwrap();let c=d.combat.as_ref().unwrap();
    small=caps.clone();small.combat_cards=1;assert_eq!(encode_run(&d,None,&small),Err(RunSessionError::Capacity{group:"combat_cards",required:c.hand.len(),capacity:1}));
    small=caps.clone();small.map_nodes=1;let map=r.public_map().unwrap();assert_eq!(encode_run(&d,Some(&map),&small),Err(RunSessionError::Capacity{group:"map_nodes",required:map.nodes.len(),capacity:1}));
}

#[test]
fn run_encoding_follows_actual_normal_start_traces_including_losses(){
    // This checks transport through real gameplay; it does not optimize the caller.
    let caps=RunCapacities::m9_default();
    for seed in [7,11]{for history in [support::M9BossHistory::AwakenedUnseen,support::M9BossHistory::TimeEaterUnseen,support::M9BossHistory::DonuDecaUnseen]{
        let report=support::complete(seed,history,8192).unwrap();
        for (index,step) in report.trace.iter().enumerate(){
            let o=encode_run(&step.decision,step.map.as_ref(),&caps).unwrap_or_else(|error|panic!("seed{seed} {history:?} step{index}: {error:?}, combat {:?}",step.decision.combat));
            assert_eq!(o.action_mask.iter().filter(|v|**v==1).count(),step.decision.candidates.len());
        }
    }}
}

#[test]
fn run_encoding_stasis_played_power_and_all_row_capacities_are_explicit(){
    let r=combat_run();let mut d=r.decision().unwrap();let caps=RunCapacities::m9_default();
    let c=d.combat.as_mut().unwrap();let card=c.hand.remove(0);let instance=card.instance;
    c.stasis.push(combat::StasisCardView{orb:c.enemies[0].instance_id.clone(),card});
    let power=c.hand.remove(0);let power_instance=power.instance;c.played_powers.push(power);
    d.candidates.clear();let o=encode_run(&d,None,&caps).unwrap();
    let cards=g(&o,"combat_cards");let occupied=cards.occupied.as_ref().unwrap().iter().filter(|v|**v==1).count();
    assert_eq!(occupied,d.combat.as_ref().unwrap().hand.len()+2);
    assert!(cards.values.chunks(8).any(|v|v[0]==5&&v[6]==0));assert!(cards.values.chunks(8).any(|v|v[0]==4));
    assert_ne!(instance,power_instance);assert!(cards.values.iter().all(|v|*v<1000));

    let mut fixtures:Vec<(&str,PublicDecision,Option<map::PublicMap>,RunCapacities,usize)>=vec![];
    let mut fixture=d.clone();let mut other=fixture.combat.as_ref().unwrap().enemies[0].clone();other.instance_id=combat::EnemyHandle::parse("supplied-second").unwrap();fixture.combat.as_mut().unwrap().enemies.push(other);let mut small=caps.clone();small.enemies=1;fixtures.push(("enemies",fixture,None,small,2));
    let mut fixture=d.clone();fixture.combat.as_mut().unwrap().player.powers=vec![combat::FullPowerView{id:combat::FullPowerId("strength"),amount:1},combat::FullPowerView{id:combat::FullPowerId("dexterity"),amount:1}];fixture.combat.as_mut().unwrap().enemies.iter_mut().for_each(|e|e.powers.clear());let mut small=caps.clone();small.powers=1;fixtures.push(("powers",fixture,None,small,2));
    let mut fixture=d.clone();fixture.combat.as_mut().unwrap().inventory.as_mut().unwrap().relics=vec![spire_core::content::RelicId::Vajra,spire_core::content::RelicId::BronzeScales];let mut small=caps.clone();small.relics=1;fixtures.push(("relics",fixture,None,small,2));
    let mut small=caps.clone();small.potions=1;fixtures.push(("potions",d.clone(),None,small,3));
    let mut fixture=initial().decision().unwrap();fixture.selection=Some(PublicSelection{required:1,selected_count:0,eligible:fixture.objects[..2].to_vec()});let mut small=caps.clone();small.selection=1;fixtures.push(("selection",fixture,None,small,2));
    let mut fixture=d.clone();fixture.player.character=spire_core::content::Character::Defect;fixture.content_profile=None;fixture.combat.as_mut().unwrap().character.orbs=vec![None,None];let mut small=caps.clone();small.orbs=1;fixtures.push(("orbs",fixture,None,small,2));
    let map=r.public_map().unwrap();let edges=map.nodes.iter().map(|n|n.edges.len()).sum();let mut small=caps.clone();small.map_edges=1;fixtures.push(("map_edges",d.clone(),Some(map),small,edges));
    for (group,fixture,map,small,required) in fixtures{assert_eq!(encode_run(&fixture,map.as_ref(),&small),Err(RunSessionError::Capacity{group:match group{"enemies"=>"enemies","powers"=>"powers","relics"=>"relics","potions"=>"potions","selection"=>"selection","orbs"=>"orbs",_=>"map_edges"},required,capacity:1}));}
}
