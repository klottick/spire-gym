use super::super::*;
use crate::combat::model::OwnedCard;
use crate::content::{CardColor, CardId, CardRarity, CardType, Catalog};

fn id(token:&str)->CardId{
    Catalog::standard().cards().iter().find(|row|row.id.as_str()==token)
        .unwrap_or_else(||panic!("missing Colorless direct card token {token}")).id
}

fn combat(token:&str,upgrades:u32)->Combat{
    let mut combat=super::full_runtime::entered_combat(3,3,|card|{
        *card=permanent_live_card(id(token),card.instance,upgrades);
    });
    combat.state.enemies[0].hp=500;
    combat.state.enemies[0].max_hp=500;
    combat
}

fn multi_combat(token:&str,upgrades:u32)->Combat{
    let mut entry=super::m8_shapes::entry(crate::content::EncounterId::ThreeShapes);
    entry.cards=vec![
        permanent_live_card(id(token),CardInstanceId(1),upgrades),
        permanent_live_card(CardId::DefendGreen,CardInstanceId(2),0),
        permanent_live_card(CardId::DefendGreen,CardInstanceId(3),0),
    ];
    entry.opening_draw=3;
    Combat::enter(entry).unwrap()
}

fn keep_only_hand_card(combat:&mut Combat,instance:u64){
    let old=std::mem::take(&mut combat.state.piles.hand);
    for card in old{
        if card.handle().as_str()==instance.to_string(){combat.state.piles.hand.push(card)}
        else{combat.state.piles.discard.push(card)}
    }
}

fn add_draw_defends(combat:&mut Combat,instances:std::ops::RangeInclusive<u64>){
    for instance in instances{
        let live=permanent_live_card(CardId::DefendGreen,CardInstanceId(instance),0);
        combat.state.card_ids_issued.insert(card_state::handle(live.instance));
        combat.state.piles.draw.push(OwnedCard::Full(live));
    }
}

fn replace_hand_card(combat:&mut Combat,instance:u64,id:CardId,upgrades:u32){
    let owned=combat.state.piles.hand.iter_mut().find(|card|card.handle().as_str()==instance.to_string())
        .unwrap_or_else(||panic!("missing hand card {instance}"));
    *owned=OwnedCard::Full(permanent_live_card(id,CardInstanceId(instance),upgrades));
}

fn full_power(combat:&Combat,enemy:usize,id:&str)->i32{
    combat.state.enemies[enemy].full_powers.iter().find(|power|power.id.0==id).map_or(0,|power|power.state.amount)
}

fn player_full_power(combat:&Combat,id:&str)->i32{
    combat.state.player_full_powers.iter().find(|power|power.id.0==id).map_or(0,|power|power.state.amount)
}

#[test]
fn m15_colorless_direct_tokens_register_with_exact_static_identity_and_flags() {
    let expected = [
        ("blind", CardType::Skill, CardRarity::Uncommon, false, false),
        ("dark_shackles", CardType::Skill, CardRarity::Uncommon, true, false),
        ("deep_breath", CardType::Skill, CardRarity::Uncommon, false, false),
        ("dramatic_entrance", CardType::Attack, CardRarity::Uncommon, true, true),
        ("finesse", CardType::Skill, CardRarity::Uncommon, false, false),
        ("flash_of_steel", CardType::Attack, CardRarity::Uncommon, false, false),
        ("good_instincts", CardType::Skill, CardRarity::Uncommon, false, false),
        ("impatience", CardType::Skill, CardRarity::Uncommon, false, false),
        ("mind_blast", CardType::Attack, CardRarity::Uncommon, false, true),
        ("panacea", CardType::Skill, CardRarity::Uncommon, true, false),
        ("panic_button", CardType::Skill, CardRarity::Uncommon, true, false),
        ("swift_strike", CardType::Attack, CardRarity::Uncommon, false, false),
        ("trip", CardType::Skill, CardRarity::Uncommon, false, false),
        ("master_of_strategy", CardType::Skill, CardRarity::Rare, true, false),
    ];

    for (index, (token, card_type, rarity, exhaust, innate)) in expected.into_iter().enumerate() {
        let definition = Catalog::standard().cards().iter()
            .find(|row| row.id.as_str() == token)
            .unwrap_or_else(|| panic!("missing Colorless direct card token {token}"));
        assert_eq!(definition.color, CardColor::Colorless, "{token} color");
        assert_eq!(definition.card_type, card_type, "{token} type");
        assert_eq!(definition.rarity, rarity, "{token} rarity");
        assert!(cards::requires_draw_history(definition.id), "{token} enables the M15 history feature");
        assert!(!definition.definition_only, "{token} is a supported direct card");

        for upgrades in 0..=1 {
            let card = card_state::permanent(definition.id, CardInstanceId(index as u64 + 1), upgrades);
            assert_eq!(card.local.exhaust_on_finish, exhaust, "{token}+{upgrades} permanent exhaust flag");
            assert_eq!(card.local.innate, innate, "{token}+{upgrades} permanent innate flag");
            let generated=card_state::generated(definition.id,CardInstanceId(100+index as u64),upgrades)
                .unwrap_or_else(||panic!("missing generated constructor for {token}"));
            assert_eq!(generated.local.exhaust_on_finish,exhaust,"{token}+{upgrades} generated exhaust flag");
            assert_eq!(generated.local.innate,innate,"{token}+{upgrades} generated innate flag");
        }
    }
}

#[test]
fn m15_colorless_direct_history_entitlement_is_feature_gated(){
    let mut fight=super::full_runtime::entered_combat(3,3,|_|{});
    fight.state.enemies[0].full_powers.push(FullPowerEntry{id:FullPowerId("shackled"),state:PowerState{amount:9,skip_first:false,monster_source_grace:false}});
    fight.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(full_power(&fight,0,"strength"),0,"legacy combat without an M15 card does not restore supplied Shackled state");
    assert_eq!(full_power(&fight,0,"shackled"),9,"legacy combat leaves supplied Shackled state untouched");
}

#[test]
fn m15_colorless_direct_public_values_track_both_upgrade_levels(){
    let rows=[
        ("blind",None,None,Some(2)),
        ("dark_shackles",None,None,Some(9)),
        ("deep_breath",None,None,Some(1)),
        ("dramatic_entrance",Some(8),None,None),
        ("finesse",None,Some(2),Some(1)),
        ("flash_of_steel",Some(3),None,Some(1)),
        ("good_instincts",None,Some(6),None),
        ("impatience",None,None,Some(2)),
        ("mind_blast",Some(17),None,None),
        ("panacea",None,None,Some(1)),
        ("panic_button",None,Some(30),Some(2)),
        ("swift_strike",Some(7),None,None),
        ("trip",None,None,Some(2)),
        ("master_of_strategy",None,None,Some(3)),
    ];
    for (token,damage,block,magic) in rows{
        for upgrades in 0..=1{
            let mut card=card_state::permanent(id(token),CardInstanceId(77),upgrades);
            if token=="mind_blast"{card.local.base_damage=Some(17);}
            let values=cards::public_values(&card).unwrap_or_else(||panic!("missing public values for {token}"));
            let (want_damage,want_block,want_magic)=match token{
                "dark_shackles"=>(None,None,Some(if upgrades>0{15}else{9})),
                "deep_breath"=>(None,None,Some(if upgrades>0{2}else{1})),
                "dramatic_entrance"=>(Some(if upgrades>0{12}else{8}),None,None),
                "finesse"=>(None,Some(if upgrades>0{4}else{2}),Some(1)),
                "flash_of_steel"=>(Some(if upgrades>0{6}else{3}),None,Some(1)),
                "good_instincts"=>(None,Some(if upgrades>0{9}else{6}),None),
                "impatience"=>(None,None,Some(if upgrades>0{3}else{2})),
                "panacea"=>(None,None,Some(if upgrades>0{2}else{1})),
                "panic_button"=>(None,Some(if upgrades>0{40}else{30}),Some(2)),
                "swift_strike"=>(Some(if upgrades>0{10}else{7}),None,None),
                "master_of_strategy"=>(None,None,Some(if upgrades>0{4}else{3})),
                _=>(damage,block,magic),
            };
            assert_eq!((values.base_damage,values.base_block,values.magic_number),(want_damage,want_block,want_magic),"{token}+{upgrades}");
            assert_eq!(values.cost_kind,CardCostKind::Fixed,"{token}+{upgrades} cost kind");

            let fight=combat(token,upgrades);
            let mut cost_card=card_state::permanent(id(token),CardInstanceId(78),upgrades);
            if token=="mind_blast"{cost_card.local.base_damage=Some(17);}
            assert_eq!(cards::visible_cost(&fight.state,&cost_card),Some(if token=="mind_blast"&&upgrades==0{2}else if token=="mind_blast"{1}else{0}),"{token}+{upgrades} cost");
        }
    }
}

#[test]
fn m15_colorless_direct_blind_and_trip_change_target_shape_on_upgrade(){
    for (token,power) in [("blind","weak"),("trip","vulnerable")]{
        for upgrades in 0..=1{
            let mut fight=multi_combat(token,upgrades);
            let before=fight.clone();
            let wrong=if upgrades==0{None}else{Some(TargetId(2))};
            assert_eq!(fight.step_full(&CombatAction::PlayCard{instance:1,target:wrong}),Err(crate::run::RunError::InvalidAction));
            assert_eq!(fight,before,"{token}+{upgrades} rejects the opposite target shape atomically");
            let target=if upgrades==0{Some(TargetId(2))}else{None};
            fight.step_full(&CombatAction::PlayCard{instance:1,target}).unwrap();
            for (index,enemy) in fight.state.enemies.iter().enumerate(){
                let amount=enemy.full_powers.iter().find(|entry|entry.id.0==power).map_or(0,|entry|entry.state.amount);
                assert_eq!(amount,if upgrades>0||index==1{2}else{0},"{token}+{upgrades} enemy{index}");
            }
        }
    }
}

#[test]
fn m15_colorless_direct_dramatic_entrance_hits_all_live_enemies_and_exhausts(){
    for (upgrades,damage) in [(0,8),(1,12)]{
        let mut fight=multi_combat("dramatic_entrance",upgrades);
        let before=fight.state.enemies.iter().map(|enemy|enemy.hp).collect::<Vec<_>>();
        fight.step_full(&CombatAction::PlayCard{instance:1,target:None}).unwrap();
        assert_eq!(fight.state.enemies.iter().map(|enemy|enemy.hp).collect::<Vec<_>>(),before.iter().map(|hp|hp-damage).collect::<Vec<_>>());
        assert!(fight.state.piles.exhaust.iter().any(|card|card.full().is_some_and(|card|card.id==id("dramatic_entrance"))));
        assert!(engine::validate_runtime(&fight.state).is_ok());
    }
}

#[test]
fn m15_colorless_direct_finesse_and_flash_keep_card_effect_order_and_shared_modifiers(){
    for upgrades in 0..=1{
        let mut finesse=combat("finesse",upgrades);
        keep_only_hand_card(&mut finesse,3);
        finesse.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("dexterity"),state:PowerState{amount:2,skip_first:false,monster_source_grace:false}});
        let prep=cards::prepare(&finesse,&CombatAction::PlayCard{instance:3,target:None}).unwrap();
        assert_eq!(prep.effects[0].kind,EffectKind::GainBlock(if upgrades>0{6}else{4}));
        assert_eq!(prep.effects[1].kind,EffectKind::Draw(1));
        finesse.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
        assert_eq!(finesse.state.player.block,if upgrades>0{6}else{4});
        assert_eq!(finesse.state.piles.hand.len(),1,"Finesse draws through the shared discard recycle");

        let mut flash=combat("flash_of_steel",upgrades);
        keep_only_hand_card(&mut flash,3);
        flash.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("strength"),state:PowerState{amount:2,skip_first:false,monster_source_grace:false}});
        flash.state.enemies[0].full_powers.push(FullPowerEntry{id:FullPowerId("vulnerable"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
        let prep=cards::prepare(&flash,&CombatAction::PlayCard{instance:3,target:Some(TargetId(1))}).unwrap();
        assert_eq!(prep.effects[0].kind,EffectKind::Damage(DamagePayload::Fixed(if upgrades>0{12}else{7})));
        assert_eq!(prep.effects[1].kind,EffectKind::Draw(1));
        flash.step_full(&CombatAction::PlayCard{instance:3,target:Some(TargetId(1))}).unwrap();
        assert_eq!(flash.state.enemies[0].hp,if upgrades>0{488}else{493});
        assert_eq!(flash.state.piles.hand.len(),1,"Flash of Steel draws through the shared discard recycle");
    }
}

#[test]
fn m15_colorless_direct_good_instincts_and_swift_strike_use_shared_card_modifiers(){
    for upgrades in 0..=1{
        let mut block=combat("good_instincts",upgrades);
        let dexterity=2;
        block.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("dexterity"),state:PowerState{amount:dexterity,skip_first:false,monster_source_grace:false}});
        block.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
        assert_eq!(block.state.player.block,if upgrades>0{11}else{8});

        let mut strike=combat("swift_strike",upgrades);
        strike.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("strength"),state:PowerState{amount:2,skip_first:false,monster_source_grace:false}});
        strike.state.enemies[0].full_powers.push(FullPowerEntry{id:FullPowerId("vulnerable"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
        strike.step_full(&CombatAction::PlayCard{instance:3,target:Some(TargetId(1))}).unwrap();
        assert_eq!(strike.state.enemies[0].hp,if upgrades>0{482}else{487});
    }
}

#[test]
fn m15_colorless_direct_deep_breath_recycles_only_nonempty_discard_and_draws(){
    for upgrades in 0..=1{
        let mut fight=combat("deep_breath",upgrades);
        keep_only_hand_card(&mut fight,3);
        let shuffle_before=fight.state.rng.shuffle.clone();
        let initial_draw=fight.state.piles.draw.clone();
        let initial_discard=fight.state.piles.discard.clone();
        let (recycled,after_recycle)=super::super::rng::shuffle(&shuffle_before,&initial_discard).unwrap();
        let mut combined=initial_draw;
        combined.extend(recycled);
        let (mut expected_draw,expected_shuffle)=super::super::rng::shuffle(&after_recycle,&combined).unwrap();
        let prep=cards::prepare(&fight,&CombatAction::PlayCard{instance:3,target:None}).unwrap();
        assert_eq!(prep.effects.len(),3,"Deep Breath requests a second shuffle of the combined draw pile");
        assert_eq!(prep.effects[0].kind,EffectKind::Recycle);
        assert_eq!(prep.effects[1].kind,EffectKind::ShuffleDraw);
        let draw_count=if upgrades>0{2}else{1};
        assert_eq!(prep.effects[2].kind,EffectKind::Draw(draw_count));
        let source=Source::Card(CardHandle::parse("3").unwrap());
        assert!(prep.effects.iter().all(|effect|effect.source==source&&effect.target.is_none()),
            "both shuffle requests and the following draw retain the card owner");
        let mut expected_hand=Vec::new();
        for _ in 0..draw_count{expected_hand.push(expected_draw.pop().unwrap().handle());}
        fight.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
        assert_eq!(fight.state.rng.shuffle,expected_shuffle,"both named shuffle requests advance the shared cursor in order");
        assert_eq!(fight.state.piles.draw,expected_draw,"the explicit shuffle permutes the combined existing draw and recycled discard");
        assert_eq!(fight.state.piles.hand.iter().map(OwnedCard::handle).collect::<Vec<_>>(),expected_hand);
        assert_eq!(fight.state.piles.hand.len(),if upgrades>0{2}else{1});
        assert_eq!(fight.state.piles.discard.len(),1,"only Deep Breath remains in discard after recycle");
        let owners=fight.state.piles.all(None).map(OwnedCard::handle).collect::<Vec<_>>();
        assert_eq!(owners.len(),fight.state.card_ids_issued.len());
        assert_eq!(owners.iter().collect::<std::collections::BTreeSet<_>>().len(),owners.len());
    }

    let mut empty=combat("deep_breath",0);
    let shuffle_before=empty.state.rng.shuffle.clone();
    let prep=cards::prepare(&empty,&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    assert_eq!(prep.effects.len(),1,"empty discard skips Recycle");
    assert_eq!(prep.effects[0].kind,EffectKind::Draw(1));
    empty.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    assert_eq!(empty.state.rng.shuffle,shuffle_before,"empty discard consumes no shuffle words");
}

#[test]
fn m15_colorless_direct_master_strategy_and_impatience_use_real_draw_and_hand_state(){
    for upgrades in 0..=1{
        let mut master=combat("master_of_strategy",upgrades);
        keep_only_hand_card(&mut master,3);
        let prep=cards::prepare(&master,&CombatAction::PlayCard{instance:3,target:None}).unwrap();
        assert_eq!(prep.effects[0].kind,EffectKind::Draw(if upgrades>0{4}else{3}));
        master.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
        assert_eq!(master.state.piles.hand.len(),2,"shared draw owner caps at the available cards");
        assert!(master.state.piles.exhaust.iter().any(|card|card.handle().as_str()=="3"));

        let mut attack_in_hand=combat("impatience",upgrades);
        add_draw_defends(&mut attack_in_hand,4..=6);
        let revealed=attack_in_hand.state.public.revealed_order.len();
        attack_in_hand.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
        assert_eq!(attack_in_hand.state.piles.hand.len(),2,"Impatience remains playable when an Attack is in hand");
        assert_eq!(attack_in_hand.state.public.revealed_order.len(),revealed,"the resolution-time Attack scan suppresses its draw");
        assert_eq!(attack_in_hand.state.piles.draw.len(),3);

        let mut no_attack=combat("impatience",upgrades);
        replace_hand_card(&mut no_attack,1,CardId::DefendGreen,0);
        add_draw_defends(&mut no_attack,4..=(if upgrades>0{6}else{5}));
        let revealed=no_attack.state.public.revealed_order.len();
        no_attack.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
        assert_eq!(no_attack.state.piles.hand.len(),2+if upgrades>0{3}else{2});
        assert_eq!(no_attack.state.public.revealed_order.len(),revealed+if upgrades>0{3}else{2});
    }
}

#[test]
fn m15_colorless_direct_mind_blast_captures_draw_count_on_hand_entry(){
    for upgrades in 0..=1{
        let mut entry=super::m8_shapes::entry(crate::content::EncounterId::ThreeShapes);
        entry.cards=vec![
            permanent_live_card(id("mind_blast"),CardInstanceId(1),upgrades),
            permanent_live_card(CardId::DefendGreen,CardInstanceId(2),0),
            permanent_live_card(CardId::DefendGreen,CardInstanceId(3),0),
            permanent_live_card(CardId::DefendGreen,CardInstanceId(4),0),
        ];
        entry.opening_draw=1;
        let mut fight=Combat::enter(entry).unwrap();
        let (instance,captured)=fight.state.piles.hand.iter().find_map(|owned|owned.full().filter(|card|card.id==id("mind_blast")))
            .map(|card|(card.instance.0,card.local.base_damage.expect("draw count is stored on hand entry")))
            .expect("innate Mind Blast enters the opening hand");
        assert_eq!(captured,fight.state.piles.draw.len() as i32);
        assert!(captured>0);
        fight.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),
            kind:EffectKind::Draw(1)}],None).unwrap();
        let refreshed=fight.state.piles.hand.iter().filter_map(OwnedCard::full)
            .find(|card|card.id==id("mind_blast")).expect("Mind Blast remains in hand after another card is drawn");
        let refreshed_value=refreshed.local.base_damage.expect("shared hand refresh recalculates the retained card");
        assert_eq!(refreshed_value,fight.state.piles.draw.len() as i32);
        assert_eq!(refreshed_value,captured-1);
        if let Some(card)=fight.state.piles.draw.pop(){fight.state.piles.discard.push(card);}
        assert_ne!(fight.state.piles.draw.len() as i32,refreshed_value,"a later use does not live-query the draw pile");
        let prepared=cards::prepare(&fight,&CombatAction::PlayCard{instance,target:Some(TargetId(1))}).unwrap();
        assert_eq!(prepared.effects[0].kind,EffectKind::Damage(DamagePayload::Fixed(refreshed_value)));
    }
}

#[test]
fn m15_colorless_direct_panacea_adds_artifact_without_cleaning_existing_debuffs(){
    for upgrades in 0..=1{
        let mut fight=combat("panacea",upgrades);
        fight.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("weak"),state:PowerState{amount:2,skip_first:false,monster_source_grace:false}});
        fight.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
        assert_eq!(player_full_power(&fight,"artifact"),if upgrades>0{2}else{1});
        assert_eq!(player_full_power(&fight,"weak"),2,"Panacea does not remove already-applied debuffs");
        assert!(fight.state.piles.exhaust.iter().any(|card|card.handle().as_str()=="3"));
    }
}

#[test]
fn m15_colorless_direct_dark_shackles_uses_source_gated_shared_shackled_and_artifact(){
    for upgrades in 0..=1{
        let amount=if upgrades>0{15}else{9};
        for artifact in 0..=2{
            let mut fight=combat("dark_shackles",upgrades);
            fight.state.enemies[0].full_powers.push(FullPowerEntry{id:FullPowerId("strength"),state:PowerState{amount:4,skip_first:false,monster_source_grace:false}});
            if artifact>0{fight.state.enemies[0].full_powers.push(FullPowerEntry{id:FullPowerId("artifact"),state:PowerState{amount:artifact,skip_first:false,monster_source_grace:false}});}
            let action=CombatAction::PlayCard{instance:3,target:Some(TargetId(1))};
            let prepared=cards::prepare(&fight,&action).unwrap();
            assert_eq!(prepared.effects.iter().filter(|effect|effect.kind==EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:-amount}).count(),1,
                "Dark Shackles always queues its strength request; artifact={artifact}");
            assert_eq!(prepared.effects.iter().filter(|effect|effect.kind==EffectKind::ApplyFullPower{id:FullPowerId("shackled"),amount}).count(),usize::from(artifact==0),
                "restoration entitlement is gated by Artifact at use; artifact={artifact}");
            let expected_source=Source::Card(CardHandle::parse("3").unwrap());
            let expected_target=Some(Actor::Enemy(fight.state.enemies[0].instance_id.clone()));
            assert!(prepared.effects.iter().all(|effect|effect.source==expected_source&&effect.target==expected_target),
                "both applications retain the selected card and enemy owners");

            fight.step_full(&action).unwrap();
            assert!(fight.state.silent.history_enabled,"playing an M15 Colorless card enables shared history semantics");
            assert_eq!(full_power(&fight,0,"strength"),if artifact==0{4-amount}else{4},"strength result with Artifact charge {artifact}");
            assert_eq!(full_power(&fight,0,"artifact"),if artifact>0{artifact-1}else{0},"Artifact consumes one charge only for the debuff request");
            assert_eq!(full_power(&fight,0,"shackled"),if artifact==0{amount}else{0},"Shackled is granted only by the use-time gate");
            fight.step_full(&CombatAction::EndTurn).unwrap();
            assert_eq!(full_power(&fight,0,"strength"),4,"shared Shackled end-turn restore applies only to its captured entitlement");
            assert_eq!(full_power(&fight,0,"shackled"),0);
            assert_eq!(full_power(&fight,0,"artifact"),if artifact>0{artifact-1}else{0});
        }
    }
}

#[test]
fn m15_colorless_direct_panic_button_blocks_card_block_but_not_direct_block_or_entrench(){
    let mut fight=combat("panic_button",0);
    fight.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    assert_eq!(fight.state.player.block,30);
    assert_eq!(player_full_power(&fight,"no_block"),2);

    replace_hand_card(&mut fight,2,CardId::GoodInstincts,0);
    replace_hand_card(&mut fight,1,CardId::Entrench,0);
    fight.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::GainBlock(5)}],None).unwrap();
    assert_eq!(fight.state.player.block,35,"direct Block is not a card Block modifier");
    fight.step_full(&CombatAction::PlayCard{instance:2,target:None}).unwrap();
    assert_eq!(fight.state.player.block,35,"No Block suppresses Good Instincts card Block");
    fight.step_full(&CombatAction::PlayCard{instance:1,target:None}).unwrap();
    assert_eq!(fight.state.player.block,70,"Entrench doubles existing Block through its direct effect owner");
    fight.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(player_full_power(&fight,"no_block"),1);
    fight.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(player_full_power(&fight,"no_block"),0,"the two-turn No Block power expires through the shared power owner");
}

#[test]
fn m15_colorless_hand_of_greed_uses_existing_currency_custody_and_fatal_gate() {
    for upgrades in 0..=1 {
        for minion in [false,true] {
            let mut c=combat("hand_of_greed",upgrades);
            c.set_thief_gold_for_entry(99).unwrap();
            c.state.enemies[0].hp=1;
            if minion { c.state.enemies[0].full_powers.push(FullPowerEntry {
                id:FullPowerId("minion"),state:PowerState { amount:1,skip_first:false,monster_source_grace:false }
            }); }
            c.step_full(&CombatAction::PlayCard { instance:3,target:Some(TargetId(1)) }).unwrap();
            assert_eq!(c.current_combat_gold(),Some(99+if minion { 0 } else { 20+5*upgrades }));
            assert_eq!(c.public_full().unwrap().outcome,Some(Outcome::Victory));
            assert!(!c.state.piles.exhaust.iter().any(|card|card.handle().as_str()=="3"),"HandOfGreed has no Exhaust flag; terminal cleanup owns final card location");
            let before=c.snapshot();
            assert!(c.step_full(&CombatAction::PlayCard { instance:3,target:Some(TargetId(1)) }).is_err());
            assert_eq!(c.snapshot(),before,"finished Fatal reward cannot be replayed");
        }
    }
}

#[test]
fn m15_colorless_hand_of_greed_excludes_real_half_death_and_retains_overflow_prefix() {
    for (encounter,slot) in [(crate::content::EncounterId::ThreeDarklings,0),(crate::content::EncounterId::AwakenedOne,2)] {
        let mut entry=super::m8_shapes::entry(encounter);
        entry.cards=vec![permanent_live_card(id("hand_of_greed"),CardInstanceId(3),0)];
        entry.opening_draw=1;
        let mut c=Combat::enter(entry).unwrap();
        c.set_thief_gold_for_entry(99).unwrap();c.state.enemies[slot].hp=1;
        c.step_full(&CombatAction::PlayCard { instance:3,target:Some(TargetId(slot as u64+1)) }).unwrap();
        assert_eq!(c.state.enemies[slot].life,ActorLife::HalfDead);
        assert_eq!(c.current_combat_gold(),Some(99));
    }
    let mut overflow=combat("hand_of_greed",0);
    overflow.set_thief_gold_for_entry(u32::MAX).unwrap();overflow.state.enemies[0].hp=1;
    let error=overflow.step_full(&CombatAction::PlayCard { instance:3,target:Some(TargetId(1)) }).unwrap_err();
    assert!(matches!(error,crate::run::RunError::Faulted(_)));
    assert_eq!(overflow.state.enemies[0].hp,0,"lethal hit remains accepted before gold overflow");
    assert_eq!(overflow.state.player.energy,1);
    assert_eq!(overflow.current_combat_gold(),Some(u32::MAX));
    let before=overflow.snapshot();
    assert_eq!(overflow.step_full(&CombatAction::EndTurn),Err(error));assert_eq!(overflow.snapshot(),before);
}
