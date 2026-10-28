use crate::{combat::rng::Partition,content::{CardColor,CardRarity,Catalog,CardId},run::{self,rooms::m10_content}};
use crate::combat::CombatAction;
use super::rooms::choose;

#[test]
fn m10_all_red_and_exclusive_items_have_real_producers(){
    for (rarity,expected) in [(CardRarity::Common,20),(CardRarity::Uncommon,36),(CardRarity::Rare,16)] {
        let ids=m10_content::red_candidates(rarity);
        assert_eq!(ids.len(),expected,"{rarity:?}");
        assert_eq!(ids.iter().copied().collect::<std::collections::BTreeSet<_>>().len(),expected,"duplicates in {rarity:?}");
        for id in ids {
            let row=Catalog::standard().card(*id).expect("candidate must have a definition");
            assert_eq!(row.color,CardColor::Red);
            assert_eq!(row.rarity,rarity);
            assert!(!row.definition_only,"{id:?} is definition-only");
        }
    }
    assert_eq!(m10_content::red_candidates(CardRarity::Basic).len(),3);
    assert!(!Catalog::standard().card(CardId::Bash).unwrap().definition_only,"Bash has a real full-run payload and must not remain a placeholder row");
    let ordinary=m10_content::red_candidates(CardRarity::Common).len()
        +m10_content::red_candidates(CardRarity::Uncommon).len()
        +m10_content::red_candidates(CardRarity::Rare).len();
    assert_eq!(ordinary,72);
    let generated=m10_content::infernal_candidates();
    assert_eq!(generated.len(),28);
    assert_eq!(generated.iter().copied().collect::<std::collections::BTreeSet<_>>().len(),28,"Infernal Blade support must not duplicate identities");
    assert!(!generated.contains(&CardId::Feed));
    assert!(!generated.contains(&CardId::Reaper));
    assert!(generated.iter().all(|id|Catalog::standard().card(*id).is_some_and(|row|row.color==CardColor::Red&&row.card_type==crate::content::CardType::Attack&&row.rarity!=CardRarity::Basic&&!row.definition_only)));
    let expected=m10_content::red_candidates(CardRarity::Common).iter().chain(m10_content::red_candidates(CardRarity::Uncommon)).chain(m10_content::red_candidates(CardRarity::Rare)).copied().filter(|id|*id!=CardId::Feed&&*id!=CardId::Reaper&&Catalog::standard().card(*id).is_some_and(|row|row.card_type==crate::content::CardType::Attack)).collect::<std::collections::BTreeSet<_>>();
    assert_eq!(generated.iter().copied().collect::<std::collections::BTreeSet<_>>(),expected,"the generator table must equal the entire supported ordinary red Attack set with only Feed/Reaper excluded");
    let exclusive_relics=[crate::content::RelicId::BurningBlood,crate::content::RelicId::BlackBlood,crate::content::RelicId::MagicFlower,crate::content::RelicId::RedSkull,crate::content::RelicId::PaperPhrog,crate::content::RelicId::ChampionsBelt,crate::content::RelicId::CharonsAshes,crate::content::RelicId::SelfFormingClay,crate::content::RelicId::Brimstone,crate::content::RelicId::MarkOfPain,crate::content::RelicId::RunicCube];
    assert_eq!(exclusive_relics.len(),11);
    for relic in exclusive_relics {
        assert!(Catalog::standard().relic(relic).is_some(),"missing Ironclad exclusive relic producer definition {relic:?}");
    }
    for potion in [crate::content::PotionId::BloodPotion,crate::content::PotionId::Elixir,crate::content::PotionId::HeartOfIron] {
        assert!(Catalog::standard().potion(potion).is_some(),"missing Ironclad exclusive potion producer definition {potion:?}");
    }
}

#[test]
fn m10_red_capture_tables_equal_the_independent_literal_reference_roster(){
    let expected=[
        (CardRarity::Basic,&[CardId::Strike,CardId::Defend,CardId::Bash][..]),
        (CardRarity::Common,&[CardId::Armaments,CardId::BodySlam,CardId::Clash,CardId::SwordBoomerang,CardId::WildStrike,CardId::TrueGrit,CardId::Headbutt,CardId::Warcry,CardId::Havoc,CardId::Anger,CardId::IronWave,CardId::Cleave,CardId::PommelStrike,CardId::ShrugItOff,CardId::TwinStrike,CardId::Clothesline,CardId::HeavyBlade,CardId::PerfectedStrike,CardId::Thunderclap,CardId::Flex][..]),
        (CardRarity::Uncommon,&[CardId::SearingBlow,CardId::Carnage,CardId::Hemokinesis,CardId::Pummel,CardId::Uppercut,CardId::Entrench,CardId::GhostlyArmor,CardId::Intimidate,CardId::PowerThrough,CardId::RecklessCharge,CardId::SeeingRed,CardId::Shockwave,CardId::SpotWeakness,CardId::BattleTrance,CardId::DualWield,CardId::Whirlwind,CardId::Inflame,CardId::Evolve,CardId::FireBreathing,CardId::FlameBarrier,CardId::Rage,CardId::Rupture,CardId::DarkEmbrace,CardId::FeelNoPain,CardId::Sentinel,CardId::SecondWind,CardId::SeverSoul,CardId::Metallicize,CardId::Combust,CardId::Bloodletting,CardId::BurningPact,CardId::Disarm,CardId::BloodForBlood,CardId::Rampage,CardId::Dropkick,CardId::InfernalBlade][..]),
        (CardRarity::Rare,&[CardId::Immolate,CardId::Exhume,CardId::DoubleTap,CardId::Juggernaut,CardId::Corruption,CardId::FiendFire,CardId::Barricade,CardId::Berserk,CardId::Brutality,CardId::DemonForm,CardId::Bludgeon,CardId::Feed,CardId::Reaper,CardId::Impervious,CardId::Offering,CardId::LimitBreak][..]),
    ];
    for (rarity,reference) in expected {
        let actual=m10_content::red_candidates(rarity).iter().copied().collect::<std::collections::BTreeSet<_>>();
        let reference=reference.iter().copied().collect::<std::collections::BTreeSet<_>>();
        assert_eq!(actual,reference,"captured {rarity:?} support differs from the separately declared reference roster");
    }
}

#[test]
fn every_m10_red_roster_card_is_permanently_obtainable_upgrade_checked_and_removable(){
    let mut base=super::m7_rooms::continuing();
    base.state.room_loop.as_mut().unwrap().scenario=run::RoomScenario::M10IroncladHeart;
    for rarity in [CardRarity::Basic,CardRarity::Common,CardRarity::Uncommon,CardRarity::Rare] {
        for &id in m10_content::red_candidates(rarity) {
            let mut state=base.state.clone();
            let instance=super::super::mutations::obtain_card(&mut state,id,0).unwrap_or_else(|error|panic!("{id:?} obtain: {error:?}"));
            let card=state.deck.iter().find(|card|card.instance==instance).unwrap().clone();
            assert_eq!(card.id,id);
            if super::super::mutations::can_upgrade(&state,&card) {
                super::super::mutations::upgrade_card(&mut state,instance).unwrap_or_else(|error|panic!("{id:?} upgrade: {error:?}"));
                assert_eq!(state.deck.iter().find(|card|card.instance==instance).unwrap().upgrades,1,"{id:?}");
            }
            assert_eq!(super::super::mutations::remove_card(&mut state,instance).unwrap().id,id,"{id:?} removal");
        }
    }
}

#[test]
fn every_ironclad_exclusive_relic_has_an_actual_m10_acquisition_owner(){
    let exclusives=[crate::content::RelicId::BurningBlood,crate::content::RelicId::BlackBlood,crate::content::RelicId::MagicFlower,crate::content::RelicId::RedSkull,crate::content::RelicId::PaperPhrog,crate::content::RelicId::ChampionsBelt,crate::content::RelicId::CharonsAshes,crate::content::RelicId::SelfFormingClay,crate::content::RelicId::Brimstone,crate::content::RelicId::MarkOfPain,crate::content::RelicId::RunicCube];
    for relic in exclusives {
        let mut run=super::m7_rooms::continuing();run.state.room_loop.as_mut().unwrap().scenario=run::RoomScenario::M10IroncladHeart;
        let before=run.state.relics.clone();
        if relic==crate::content::RelicId::BlackBlood {
            assert_eq!(before.first(),Some(&crate::content::RelicId::BurningBlood));
        }else if relic==crate::content::RelicId::BurningBlood {
            continue; // Supplied M10 creation already owns this actual Neow-start relic.
        }
        crate::items::acquire_relic(&mut run.state,relic).unwrap_or_else(|error|panic!("{relic:?} acquisition: {error:?}"));
        if relic==crate::content::RelicId::BlackBlood {
            assert_eq!(run.state.relics.first(),Some(&relic),"Black Blood replaces the owned starter relic");
            assert!(!run.state.relics.contains(&crate::content::RelicId::BurningBlood));
        }else{
            assert_eq!(run.state.relics.last(),Some(&relic),"{relic:?} must enter permanent Run inventory");
        }
    }
}

#[test]
fn m10_new_cards_remain_eligible_in_existing_shrine_ancient_and_falling_events(){
    let mut shrine=super::m7_rooms::continuing();
    shrine.state.room_loop.as_mut().unwrap().scenario=run::RoomScenario::M10IroncladHeart;
    let current=shrine.state.map.as_ref().unwrap().nodes.iter().find(|node|node.floor==18).unwrap().id;shrine.state.map.as_mut().unwrap().current=Some(current);shrine.state.floor=18;
    shrine.state.deck=vec![run::CardInstance{ misc:0,instance:41,id:CardId::SearingBlow,upgrades:0}];
    let mut next=shrine.next_choice;
    shrine.state.room_loop.as_mut().unwrap().screen=run::rooms::events::screen(&mut next,crate::content::EventId::UpgradeShrine).unwrap();
    shrine.next_choice=next;
    let choice=shrine.decision().unwrap().room.unwrap().choices.into_iter().find(|row|row.kind==run::RoomChoiceKind::ShrineUpgrade).unwrap().id;
    shrine.step(&run::GameAction::Choose(choice)).unwrap();
    let selected=shrine.state.pending.as_ref().unwrap().selection.candidates.iter().find(|candidate|candidate.card.instance==41).unwrap().id;
    shrine.step(&run::GameAction::Select(selected)).unwrap();
    assert_eq!(shrine.state.deck[0].upgrades,1,"the existing Upgrade Shrine accepts M10's newly supported red card");

    let mut ancient=super::m7_rooms::continuing();
    ancient.state.room_loop.as_mut().unwrap().scenario=run::RoomScenario::M10IroncladHeart;
    let current=ancient.state.map.as_ref().unwrap().nodes.iter().find(|node|node.floor==18).unwrap().id;ancient.state.map.as_mut().unwrap().current=Some(current);ancient.state.floor=18;
    ancient.state.deck=vec![run::CardInstance{ misc:0,instance:51,id:CardId::SearingBlow,upgrades:0},run::CardInstance{ misc:0,instance:52,id:CardId::Inflame,upgrades:0}];
    let mut next=ancient.next_choice;
    ancient.state.room_loop.as_mut().unwrap().screen=run::rooms::events::screen(&mut next,crate::content::EventId::AncientWriting).unwrap();
    ancient.next_choice=next;
    let choice=ancient.decision().unwrap().room.unwrap().choices.into_iter().find(|row|row.kind==run::RoomChoiceKind::AncientElegance).unwrap().id;
    ancient.step(&run::GameAction::Choose(choice)).unwrap();
    let selected=ancient.state.pending.as_ref().unwrap().selection.candidates.iter().find(|candidate|candidate.card.instance==51).unwrap().id;
    ancient.step(&run::GameAction::Select(selected)).unwrap();
    assert!(!ancient.state.deck.iter().any(|card|card.instance==51),"Ancient Writing's existing remove choice can select a new red card");

    let mut falling=super::m7_rooms::continuing();
    falling.state.room_loop.as_mut().unwrap().scenario=run::RoomScenario::M10IroncladHeart;
    falling.state.act=3;falling.state.floor=36;
    let map=falling.state.map.as_mut().unwrap();map.act=3;let node=map.nodes[0].id;map.current=Some(node);
    falling.state.deck=vec![run::CardInstance{ misc:0,instance:61,id:CardId::SearingBlow,upgrades:0},run::CardInstance{ misc:0,instance:62,id:CardId::SearingBlow,upgrades:2},run::CardInstance{ misc:0,instance:63,id:CardId::Disarm,upgrades:0},run::CardInstance{ misc:0,instance:64,id:CardId::Inflame,upgrades:0}];
    let mut next=falling.next_choice;
    let prepared=run::rooms::events::act3::prepare(&falling,crate::content::EventId::Falling,node,0,&mut next).unwrap();
    falling.next_choice=next;falling.state.rng=prepared.rng;
    let room=falling.state.room_loop.as_mut().unwrap();room.screen=prepared.screen;room.act3_event=Some(prepared.frame);
    let attack=falling.decision().unwrap().room.unwrap().choices.into_iter().find(|row|matches!(row.kind,run::RoomChoiceKind::FallingLose{card_type:crate::content::CardType::Attack,..})).unwrap();
    let run::RoomChoiceKind::FallingLose{instance:Some(instance),..}=attack.kind else{panic!("one repeated Searing Blow must be captured as the attack option")};
    assert!([61,62].contains(&instance));
    falling.step(&run::GameAction::Choose(attack.id)).unwrap();
    assert!(!falling.state.deck.iter().any(|card|card.instance==instance));
    assert!(falling.state.deck.iter().any(|card|card.id==CardId::SearingBlow),"only the captured duplicate leaves the deck");
}

#[test]
fn m10_captured_uncommon_reward_rows_sample_distinct_real_uncommons(){
    let mut rng=run::RunRng::new([91;32],Partition::Test,4);
    let before=rng.clone();
    let row=m10_content::sample_capture_row(&mut rng,&[1,17,2],&[CardRarity::Uncommon,CardRarity::Uncommon]).unwrap();
    assert_eq!(row.len(),2);
    assert_ne!(row[0],row[1]);
    assert!(row.iter().all(|id|Catalog::standard().card(*id).is_some_and(|card|card.color==CardColor::Red&&card.rarity==CardRarity::Uncommon&&!card.definition_only)));
    assert_ne!(rng,before,"capture sampling advances its owned scoped stream");
}

#[test]
fn m10_boss_card_reward_uses_three_distinct_rare_red_choices(){
    let (mut run,node)=super::m7_rooms::boss_source(crate::content::EncounterId::Champ);
    run.state.room_loop.as_mut().unwrap().scenario=run::RoomScenario::M10IroncladHeart;
    run.step(&run::GameAction::Move(node)).unwrap();
    for _ in 0..32 {
        let decision=run.decision().unwrap();
        if decision.combat.is_none(){break}
        let action=decision.candidates.into_iter().find_map(|candidate|match candidate.action{
            run::GameAction::Combat(action@CombatAction::PlayCard{target:Some(_),..})=>Some(action),_=>None
        }).unwrap_or(CombatAction::EndTurn);
        run.choose_combat(action).unwrap();
    }
    assert!(run.state.combat.is_none());
    let potion=run.decision().unwrap().room.unwrap().choices.into_iter().find_map(|choice|match choice.kind{run::RoomChoiceKind::Potion(id)=>Some(id),_=>None}).unwrap();
    assert!([crate::content::PotionId::BloodPotion,crate::content::PotionId::Elixir,crate::content::PotionId::HeartOfIron].contains(&potion),"M10 elite/boss reward must capture an exclusive Ironclad potion, got {potion:?}");
    choose(&mut run,run::RoomChoiceKind::CardReward);
    let choices=run.decision().unwrap().room.unwrap().choices.into_iter().filter_map(|choice|match choice.kind{run::RoomChoiceKind::Card{id,..}=>Some(id),_=>None}).collect::<Vec<_>>();
    assert_eq!(choices.len(),3);
    assert_eq!(choices.iter().copied().collect::<std::collections::BTreeSet<_>>().len(),3);
    assert!(choices.iter().all(|id|Catalog::standard().card(*id).is_some_and(|card|card.color==CardColor::Red&&card.rarity==CardRarity::Rare&&!card.definition_only)));
    let picked=choices[0];
    choose(&mut run,run::RoomChoiceKind::Card{id:picked,upgrades:0});
    assert!(run.state.deck.iter().any(|card|card.id==picked),"every captured red card must be permanently acquirable");
}

#[test]
fn m10_ordinary_card_reward_captures_one_common_and_two_distinct_uncommons(){
    let mut run=super::m7_rooms::continuing();
    run.state.room_loop.as_mut().unwrap().scenario=run::RoomScenario::M10IroncladHeart;
    let node=run.state.map.as_ref().unwrap().nodes.iter().find(|node|node.floor==18&&node.kind==run::MapNodeKind::Monster).unwrap().id;
    run.step(&run::GameAction::Move(node)).unwrap();
    for _ in 0..32 {
        let decision=run.decision().unwrap();
        if decision.combat.is_none(){break}
        let action=decision.candidates.into_iter().find_map(|candidate|match candidate.action{
            run::GameAction::Combat(action@CombatAction::PlayCard{target:Some(_),..})=>Some(action),_=>None
        }).unwrap_or(CombatAction::EndTurn);
        run.choose_combat(action).unwrap();
    }
    assert!(run.state.combat.is_none());
    choose(&mut run,run::RoomChoiceKind::CardReward);
    let choices=run.decision().unwrap().room.unwrap().choices.into_iter().filter_map(|choice|match choice.kind{run::RoomChoiceKind::Card{id,..}=>Some(id),_=>None}).collect::<Vec<_>>();
    assert_eq!(choices.len(),3);
    let rows=choices.iter().map(|id|Catalog::standard().card(*id).unwrap()).collect::<Vec<_>>();
    assert_eq!(rows.iter().filter(|card|card.rarity==CardRarity::Common).count(),1);
    assert_eq!(rows.iter().filter(|card|card.rarity==CardRarity::Uncommon).count(),2);
    assert_ne!(choices[1],choices[2]);
    assert!(rows.iter().all(|card|card.color==CardColor::Red&&!card.definition_only));
    let selected=choices[1];choose(&mut run,run::RoomChoiceKind::Card{id:selected,upgrades:0});
    assert!(run.state.deck.iter().any(|card|card.id==selected),"actual M10 ordinary reward choice must commit the selected permanent red card");
}

#[test]
fn m10_boss_relic_choices_include_owned_and_ineligible_exclusive_options_disabled(){
    let mut run=super::m7_rewards::victory(crate::content::EncounterId::Champ);
    run.state.room_loop.as_mut().unwrap().scenario=run::RoomScenario::M10IroncladHeart;
    crate::items::acquire_relic(&mut run.state,crate::content::RelicId::BlackBlood).unwrap();
    choose(&mut run,run::RoomChoiceKind::Proceed);
    choose(&mut run,run::RoomChoiceKind::OpenChest);
    let choices=run.decision().unwrap().room.unwrap().choices;
    for relic in [crate::content::RelicId::MarkOfPain,crate::content::RelicId::RunicCube]{
        assert!(choices.iter().any(|choice|choice.kind==run::RoomChoiceKind::Relic(relic)&&choice.status==run::RoomChoiceStatus::Offered));
    }
    assert!(choices.iter().any(|choice|choice.kind==run::RoomChoiceKind::Relic(crate::content::RelicId::BlackBlood)&&choice.status==run::RoomChoiceStatus::Disabled),"{choices:?}");
    assert!(choices.iter().any(|choice|choice.kind==run::RoomChoiceKind::Relic(crate::content::RelicId::EmptyCage)&&choice.status==run::RoomChoiceStatus::Offered));
    for relic in [crate::content::RelicId::MarkOfPain,crate::content::RelicId::RunicCube] {
        let mut branch=run.clone();choose(&mut branch,run::RoomChoiceKind::Relic(relic));
        assert!(branch.state.relics.contains(&relic),"captured M10 boss choice must commit {relic:?}");
    }
}

#[test]
fn m10_shop_captures_one_card_per_rarity_brimstone_and_exclusive_potions(){
    let mut run=super::m7_rooms::continuing();
    run.state.room_loop.as_mut().unwrap().scenario=run::RoomScenario::M10IroncladHeart;
    let node=run.state.map.as_ref().unwrap().nodes.iter().find(|node|node.kind==run::MapNodeKind::Shop).unwrap().clone();
    run.state.floor=node.floor;run.state.map.as_mut().unwrap().current=Some(node.id);
    let mut next=run.next_choice;
    let mut rng=run.state.rng.clone();let shop=run::rooms::shop::screen_with_rng(&run,&mut next,&mut rng).unwrap();
    let cards=shop.choices.iter().filter_map(|row|match row.kind{run::RoomChoiceKind::Card{id,..}=>Some((id,row.price)),_=>None}).collect::<Vec<_>>();
    assert_eq!(cards.len(),3,"M10 card shop offers common/uncommon/rare");
    let rarities=cards.iter().map(|(id,_)|Catalog::standard().card(*id).unwrap().rarity).collect::<Vec<_>>();
    assert_eq!(rarities.iter().filter(|r|**r==CardRarity::Common).count(),1);
    assert_eq!(rarities.iter().filter(|r|**r==CardRarity::Uncommon).count(),1);
    assert_eq!(rarities.iter().filter(|r|**r==CardRarity::Rare).count(),1);
    for (id,price) in cards {assert_eq!(Catalog::standard().card(id).unwrap().color,CardColor::Red);assert_eq!(price,Some(match Catalog::standard().card(id).unwrap().rarity{CardRarity::Common=>50,CardRarity::Uncommon=>75,CardRarity::Rare=>150,_=>unreachable!()}));}
    assert!(shop.choices.iter().any(|row|row.kind==run::RoomChoiceKind::Relic(crate::content::RelicId::Brimstone)&&row.price==Some(150)));
    for (potion,price) in [(crate::content::PotionId::BloodPotion,50),(crate::content::PotionId::Elixir,75),(crate::content::PotionId::HeartOfIron,100)]{
        assert!(shop.choices.iter().any(|row|row.kind==run::RoomChoiceKind::Potion(potion)&&row.price==Some(price)),"{potion:?}");
    }
}

#[test]
fn m10_real_shop_move_keys_capture_rng_to_destination_not_previous_floor(){
    let mut run=super::m7_rooms::continuing();
    run.state.room_loop.as_mut().unwrap().scenario=run::RoomScenario::M10IroncladHeart;
    let shop=run.state.map.as_ref().unwrap().nodes.iter().find(|node|node.kind==run::MapNodeKind::Shop).unwrap().clone();
    super::m7_rooms::before(&mut run,shop.id);
    let before_rejection=run.clone();
    assert!(run.step(&run::GameAction::Move(run::NodeId(u64::MAX))).is_err());
    assert_eq!(run,before_rejection,"rejected preparation must not consume offer or choice entropy");
    let previous_floor=run.state.floor;
    let mut expected_rng=run.state.rng.clone();
    let expected=m10_content::sample_capture_row(&mut expected_rng,&[2, u64::from(shop.floor), shop.id.0],
        &[CardRarity::Common,CardRarity::Uncommon,CardRarity::Rare]).unwrap();
    run.step(&run::GameAction::Move(shop.id)).expect("actual legal map move enters captured shop");
    let screen=run.decision().unwrap().room.unwrap();
    let actual=screen.choices.into_iter().filter_map(|row|match row.kind{run::RoomChoiceKind::Card{id,..}=>Some(id),_=>None}).collect::<Vec<_>>();
    assert_eq!(actual,expected,"capture stream must use destination act/floor/node, not stale prior run location (floor {previous_floor})");
    assert_eq!(run.state.rng,expected_rng,"actual Move commits exactly the destination-scoped capture frontier");
}

#[test]
fn m10_shop_disables_owned_brimstone_and_exclusive_potions_when_belt_is_full(){
    let mut run=super::m7_rooms::continuing();
    run.state.room_loop.as_mut().unwrap().scenario=run::RoomScenario::M10IroncladHeart;
    run.state.relics.push(crate::content::RelicId::Brimstone);
    run.state.potions=vec![Some(crate::content::PotionId::FirePotion);3];
    let node=run.state.map.as_ref().unwrap().nodes.iter().find(|node|node.kind==run::MapNodeKind::Shop).unwrap().clone();
    run.state.floor=node.floor;run.state.map.as_mut().unwrap().current=Some(node.id);
    let mut next=run.next_choice;let mut rng=run.state.rng.clone();
    let shop=run::rooms::shop::screen_with_rng(&run,&mut next,&mut rng).unwrap();
    run.state.room_loop.as_mut().unwrap().screen=shop;
    let public=run::rooms::public(&run).unwrap();
    for item in [run::RoomChoiceKind::Relic(crate::content::RelicId::Brimstone),run::RoomChoiceKind::Potion(crate::content::PotionId::BloodPotion),run::RoomChoiceKind::Potion(crate::content::PotionId::Elixir),run::RoomChoiceKind::Potion(crate::content::PotionId::HeartOfIron)] {
        assert!(public.choices.iter().any(|row|row.kind==item&&row.status==run::RoomChoiceStatus::Disabled),"{item:?}");
    }
}

#[test]
fn m10_shop_purchase_commits_exclusive_relic_and_potion_with_exact_price(){
    let mut run=super::m7_rooms::continuing();
    run.state.room_loop.as_mut().unwrap().scenario=run::RoomScenario::M10IroncladHeart;
    run.state.gold=500;
    run.state.potions=vec![None;3];
    let node=run.state.map.as_ref().unwrap().nodes.iter().find(|node|node.kind==run::MapNodeKind::Shop).unwrap().clone();
    super::m7_rooms::before(&mut run,node.id);
    let rng_before=run.state.rng.clone();
    run.step(&run::GameAction::Move(node.id)).unwrap();
    assert_ne!(run.state.rng,rng_before,"committing the captured shop commits its owned scoped offer-sampling stream");
    let before=run.state.gold;
    let id=run.decision().unwrap().room.unwrap().choices.iter().find(|row|row.kind==run::RoomChoiceKind::Relic(crate::content::RelicId::Brimstone)).unwrap().id;
    run.step(&run::GameAction::Choose(id)).unwrap();
    assert!(run.state.relics.contains(&crate::content::RelicId::Brimstone));
    assert_eq!(run.state.gold,before-150);
    let id=run.decision().unwrap().room.unwrap().choices.iter().find(|row|row.kind==run::RoomChoiceKind::Potion(crate::content::PotionId::Elixir)).unwrap().id;
    run.step(&run::GameAction::Choose(id)).unwrap();
    assert!(run.state.potions.contains(&Some(crate::content::PotionId::Elixir)));
    assert_eq!(run.state.gold,before-225);
}

#[test]
fn m10_elite_drop_sequence_captures_nineteen_unique_owned_eligible_relics(){
    let exclusive=[crate::content::RelicId::Lantern,crate::content::RelicId::Pear,crate::content::RelicId::Mango,crate::content::RelicId::MagicFlower,crate::content::RelicId::ToyOrnithopter,crate::content::RelicId::Strawberry,crate::content::RelicId::Anchor,crate::content::RelicId::BloodVial,crate::content::RelicId::Vajra,crate::content::RelicId::OddlySmoothStone,crate::content::RelicId::BagOfPreparation,crate::content::RelicId::BagOfMarbles,crate::content::RelicId::BronzeScales,crate::content::RelicId::RedSkull,crate::content::RelicId::PaperPhrog,crate::content::RelicId::ChampionsBelt,crate::content::RelicId::CharonsAshes,crate::content::RelicId::SelfFormingClay,crate::content::RelicId::Orichalcum];
    let mut cursor=0;let mut owned=Vec::new();
    for expected in exclusive {
        let (captured,next)=run::rooms::rewards::capture_m10_elite_relic(cursor,&owned).unwrap();
        assert_eq!(captured,expected);assert!(!owned.contains(&captured));owned.push(captured);cursor=next;
    }
    assert_eq!(cursor,19);
    assert!(run::rooms::rewards::capture_m10_elite_relic(cursor,&owned).is_err(),"finite source must not refill or duplicate");
}

#[test]
fn m10_act3_elite_victory_captures_exclusive_relic_and_potion_rows(){
    let mut run=super::m7_rewards::chest();
    run.state.room_loop.as_mut().unwrap().scenario=run::RoomScenario::M10IroncladHeart;
    run.state.relics.push(crate::content::RelicId::Lantern);
    run.state.room_loop.as_mut().unwrap().m10_elite_relic_cursor=1;
    choose(&mut run,run::RoomChoiceKind::Relic(crate::content::RelicId::BlackBlood));
    choose(&mut run,run::RoomChoiceKind::Proceed);
    assert_eq!(run.state.act,3);
    assert_eq!(run.state.room_loop.as_ref().unwrap().m10_elite_relic_cursor,1,"the shared finite cursor survives the actual Act2→Act3 transition");
    run.state.relics.push(crate::content::RelicId::NeowsLament);
    run.state.relic_counters.insert(crate::content::RelicId::NeowsLament,3);
    let node=run.state.map.as_ref().unwrap().nodes.iter().find(|node|(35..=49).contains(&node.floor)&&node.kind==run::MapNodeKind::Elite).unwrap().clone();
    let parent=run.state.map.as_ref().unwrap().nodes.iter().find(|candidate|candidate.edges.contains(&node.id)).unwrap().id;
    run.state.map.as_mut().unwrap().current=Some(parent);
    run.state.map.as_mut().unwrap().nodes.iter_mut().find(|candidate|candidate.id==parent).unwrap().visited=true;
    run.state.room_loop.as_mut().unwrap().screen=run::RoomView{kind:run::RoomViewKind::Map,choices:vec![]};
    run.step(&run::GameAction::Move(node.id)).unwrap();
    for _ in 0..8 {
        let decision=run.decision().unwrap();
        if decision.combat.is_none(){break}
        let action=decision.candidates.into_iter().find_map(|candidate|match candidate.action{run::GameAction::Combat(action@CombatAction::PlayCard{target:Some(_),..})=>Some(action),_=>None}).unwrap_or(CombatAction::EndTurn);
        run.choose_combat(action).unwrap();
    }
    let room=run.decision().unwrap().room.unwrap();
    assert!(room.choices.iter().any(|choice|choice.kind==run::RoomChoiceKind::Relic(crate::content::RelicId::Pear)),"the actual Act3 elite continues at the inherited cursor and skips owned Lantern");
    assert!(room.choices.iter().any(|choice|matches!(choice.kind,run::RoomChoiceKind::Potion(crate::content::PotionId::BloodPotion|crate::content::PotionId::Elixir|crate::content::PotionId::HeartOfIron))));
    choose(&mut run,run::RoomChoiceKind::Relic(crate::content::RelicId::Pear));
    assert!(run.state.relics.contains(&crate::content::RelicId::Pear));
    assert_eq!(run.state.room_loop.as_ref().unwrap().m10_elite_relic_cursor,2,"the accepted elite claim advances the retained shared sequence once");
}

#[test]
fn m10_act1_elite_victory_uses_same_finite_stock_owner(){
    let mut profile=run::CreationProfile::experienced();profile.neow_eligible=false;
    let mut run=run::Run::new_room_scenario(run::RunConfig{character:crate::content::Character::Ironclad,ascension:0},profile,run::RunRng::new([44;32],Partition::Test,0),run::RoomScenario::M10IroncladHeart).unwrap();
    choose(&mut run,run::RoomChoiceKind::NeowLament);
    run.state.relic_counters.insert(crate::content::RelicId::NeowsLament,3);
    let node=run.state.map.as_ref().unwrap().nodes.iter().find(|node|(1..=15).contains(&node.floor)&&node.kind==run::MapNodeKind::Elite).unwrap().clone();
    let parent=run.state.map.as_ref().unwrap().nodes.iter().find(|candidate|candidate.edges.contains(&node.id)).unwrap().id;
    run.state.map.as_mut().unwrap().current=Some(parent);
    run.state.map.as_mut().unwrap().nodes.iter_mut().find(|candidate|candidate.id==parent).unwrap().visited=true;
    run.step(&run::GameAction::Move(node.id)).unwrap();
    for _ in 0..8 {
        let decision=run.decision().unwrap();if decision.combat.is_none(){break}
        let action=decision.candidates.into_iter().find_map(|candidate|match candidate.action{run::GameAction::Combat(action@CombatAction::PlayCard{target:Some(_),..})=>Some(action),_=>None}).unwrap_or(CombatAction::EndTurn);
        run.choose_combat(action).unwrap();
    }
    let room=run.decision().unwrap().room.unwrap();
    assert!(room.choices.iter().any(|choice|choice.kind==run::RoomChoiceKind::Relic(crate::content::RelicId::Lantern)),"{:?}",room.choices);
    assert_eq!(run.state.room_loop.as_ref().unwrap().m10_elite_relic_cursor,1);
}

#[test]
fn m10_act2_elite_victory_uses_the_shared_finite_stock_cursor(){
    let mut run=super::m7_rooms::continuing();
    run.state.room_loop.as_mut().unwrap().scenario=run::RoomScenario::M10IroncladHeart;
    run.state.relics.push(crate::content::RelicId::NeowsLament);run.state.relic_counters.insert(crate::content::RelicId::NeowsLament,3);
    let node=run.state.map.as_ref().unwrap().nodes.iter().find(|node|(18..=33).contains(&node.floor)&&node.kind==run::MapNodeKind::Elite).unwrap().clone();
    let parent=run.state.map.as_ref().unwrap().nodes.iter().find(|candidate|candidate.edges.contains(&node.id)).unwrap().id;
    run.state.map.as_mut().unwrap().current=Some(parent);run.state.map.as_mut().unwrap().nodes.iter_mut().find(|candidate|candidate.id==parent).unwrap().visited=true;
    run.state.room_loop.as_mut().unwrap().screen=run::RoomView{kind:run::RoomViewKind::Map,choices:vec![]};
    run.step(&run::GameAction::Move(node.id)).unwrap();
    for _ in 0..8 {
        let decision=run.decision().unwrap();if decision.combat.is_none(){break}
        let action=decision.candidates.into_iter().find_map(|candidate|match candidate.action{run::GameAction::Combat(action@CombatAction::PlayCard{target:Some(_),..})=>Some(action),_=>None}).unwrap_or(CombatAction::EndTurn);
        run.choose_combat(action).unwrap();
    }
    let room=run.decision().unwrap().room.unwrap();
    assert!(room.choices.iter().any(|choice|choice.kind==run::RoomChoiceKind::Relic(crate::content::RelicId::Lantern)),"{:?}",room.choices);
    assert_eq!(run.state.room_loop.as_ref().unwrap().m10_elite_relic_cursor,1);
}

#[test]
fn m10_act3_selected_heart_boss_enters_the_actual_boss_encounter(){
    let(mut run,node)=super::m9_boss::source(crate::content::EncounterId::AwakenedOne);
    run.state.room_loop.as_mut().unwrap().scenario=run::RoomScenario::M10IroncladHeart;
    let entered=run.state.combat_progress.entered;
    run.step(&run::GameAction::Move(node)).expect("M10 must carry the captured Act3 boss through the shared actual encounter owner");
    assert_eq!((run.state.act,run.state.floor,run.state.active_combat.as_ref().unwrap().encounter),(3,50,crate::content::EncounterId::AwakenedOne));
    assert_eq!(run.state.combat_progress.entered,entered+1);
}
