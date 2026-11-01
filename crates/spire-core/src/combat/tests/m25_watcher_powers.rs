//! M25 Watcher power and turn-control cards (plan Task C4).
//! Expected faces are literals from design §5.1 and
//! docs/research/2026-10-19-act1-content-watcher.md:87,100-104,121,123-124,
//! 127,132,139,142-143,153,485-507; card-specific notes are
//! data/cards/implementation-notes.json:23-26,44-48,83-84,103-106.
//! Turn-ending compositions use the real A0 setup owner and step_full.
use super::super::*;
use crate::{content::{CardId,Character,EncounterId},items::CombatItemContext};
use super::m8_shapes::entry;
use CardId::*;

fn handle(id:u64)->CardHandle{card_state::handle(CardInstanceId(id))}
fn watcher(e:CombatEntry,deck:&[(CardId,u32)],energy:i32)->Combat{watcher_with_relics(e,deck,energy,vec![])}
fn watcher_with_relics(e:CombatEntry,deck:&[(CardId,u32)],energy:i32,relics:Vec<crate::content::RelicId>)->Combat{
    let opening_draw=deck.len().min(10);
    watcher_with_relics_opening_draw(e,deck,energy,opening_draw,relics)
}
fn watcher_with_relics_opening_draw(mut e:CombatEntry,deck:&[(CardId,u32)],energy:i32,opening_draw:usize,relics:Vec<crate::content::RelicId>)->Combat{
    e.setup.character=Character::Watcher;
    e.cards=deck.iter().enumerate().map(|(i,(id,up))|m25_watcher_live_card(*id,CardInstanceId(i as u64+1),*up).unwrap()).collect();
    e.opening_draw=opening_draw;e.setup.player.energy=energy;e.setup.player.max_energy=energy;e.setup.player.draw_per_turn=1;
    let mut items=Some(CombatItemContext::new(relics,vec![None;3]).with_m25_profile());
    let combat=Combat::enter_with_item_context(e,&mut items).unwrap();assert!(items.is_none());combat
}
fn time_eater(count:i32,intent:&'static str)->CombatEntry{
    let mut e=entry(EncounterId::TimeEater);
    e.setup.enemies[0].powers.iter_mut().find(|p|p.id.0=="time_warp").unwrap().state.amount=count;
    e.setup.enemies[0].intent=FullIntentId::from_name(intent);e
}
fn heart_entry()->CombatEntry{
    enemies::setup(EncounterId::CorruptHeart,4,0,&mut crate::run::RunRng::new([29;32],rng::Partition::Test,0),enemies::SetupContext{
        player:PlayerState{hp:300,max_hp:300,block:0,energy:3,max_energy:3,draw_per_turn:1,powers:vec![]},
        player_powers:vec![],character:Character::Watcher,cards:vec![permanent_live_card(DefendPurple,CardInstanceId(1),0)],opening_draw:1,encounter_visit:0}).unwrap()
}
fn play(c:&mut Combat,id:u64,target:Option<u64>){
    c.step_full(&CombatAction::PlayCard{instance:id,target:target.map(TargetId)})
        .unwrap_or_else(|e|panic!("play {id}: {e:?} legal={:?} fault={:?}",c.legal_actions_full(),c.fault_record()));
}
fn end(c:&mut Combat){c.step_full(&CombatAction::EndTurn).unwrap();}
fn player_power(c:&Combat,id:&str)->i32{c.state.player_full_powers.iter().find(|p|p.id.0==id).map_or(0,|p|p.state.amount)}
fn power(c:&Combat,slot:usize,id:&str)->i32{c.state.enemies[slot].full_powers.iter().find(|p|p.id.0==id).map_or(0,|p|p.state.amount)}
fn full_power(id:&'static str,n:i32)->FullPowerEntry{FullPowerEntry{id:FullPowerId(id),state:PowerState{amount:n,skip_first:false,monster_source_grace:false}}}
fn hooks(c:&Combat,hook:Hook)->usize{c.state.hook_events.iter().filter(|event|event.hook==hook).count()}
fn in_pile(pile:&[model::OwnedCard],id:u64)->bool{pile.iter().any(|owned|owned.handle()==handle(id))}
fn set_intent(c:&mut Combat,slot:usize,name:&'static str){
    let intent=ActorIntentId::Full(FullIntentId::from_name(name));let enemy=&mut c.state.enemies[slot];
    enemy.intent_id=intent.clone();enemy.selected_move=intent.clone();enemy.move_history.push(intent);effects::refresh_enemy_values(&mut c.state).unwrap();
}
fn place(c:&mut Combat,id:u64,zone:SelectionSourceZone){
    let h=handle(id);let take=|pile:&mut Vec<model::OwnedCard>|pile.iter().position(|owned|owned.handle()==h).map(|i|pile.remove(i));
    let owned=take(&mut c.state.piles.hand).or_else(||take(&mut c.state.piles.draw)).or_else(||take(&mut c.state.piles.discard)).unwrap();
    match zone{SelectionSourceZone::Hand=>c.state.piles.hand.push(owned),SelectionSourceZone::Draw=>c.state.piles.draw.push(owned),SelectionSourceZone::Discard=>c.state.piles.discard.push(owned),_=>unreachable!()}
    piles::refresh_hand_values(&mut c.state);
}
fn total_cards(c:&Combat,id:CardId)->usize{piles::owned_full_cards(&c.state).filter(|card|card.id==id).count()}
fn face(up:u32,base:i32,upgraded:i32)->i32{if up==0{base}else{upgraded}}

#[test]
fn m25_powers_constructor_stack_and_callbacks(){
    // Pressure Points: source research :87, :502-507; Mark stacks before all stored Marks
    // are triggered through HP_LOSS. Expected 8/11 and 16/22 are design §5.1 literals.
    for up in [0,1]{
        let mark=face(up,8,11);let mut c=watcher(entry(EncounterId::OrbWalker),&[(PressurePoints,up),(PressurePoints,up)],9);
        let before=c.state.enemies[0].hp;play(&mut c,1,Some(1));
        assert_eq!((power(&c,0,"mark"),before-c.state.enemies[0].hp),(mark,mark));
        let before=c.state.enemies[0].hp;play(&mut c,2,Some(1));
        assert_eq!((power(&c,0,"mark"),before-c.state.enemies[0].hp),(2*mark,2*mark));
        let artifact=watcher(entry(EncounterId::ThreeDarklings),&[(PressurePoints,up)],9);
        let mut blocked=artifact;blocked.state.enemies[0].full_powers.push(full_power("mark",3));
        blocked.state.enemies[1].full_powers.push(full_power("artifact",1));
        let before=blocked.state.enemies.iter().map(|enemy|enemy.hp).collect::<Vec<_>>();play(&mut blocked,1,Some(2));
        assert_eq!(power(&blocked,0,"mark"),3,"an Artifact-blocked new Mark leaves existing Mark intact");
        assert_eq!(power(&blocked,1,"mark"),0);assert_eq!(power(&blocked,1,"artifact"),0);
        assert_eq!(blocked.state.enemies[0].hp,before[0]-3,"old Marks still trigger on every enemy");
    }

    // Talk to the Hand: research :124, :494-500. Its own hit precedes installation;
    // later zero/fully-blocked multi-hits return the stacked 2/3 amount per hit.
    for up in [0,1]{
        let amount=face(up,2,3);let mut c=watcher(entry(EncounterId::OrbWalker),&[(TalkToTheHand,up),(FlyingSleeves,0)],9);
        let hp=c.state.enemies[0].hp;play(&mut c,1,Some(1));
        assert_eq!((hp-c.state.enemies[0].hp,c.state.player.block,power(&c,0,"block_return")),(face(up,5,7),0,amount));
        c.state.enemies[0].block=100;play(&mut c,2,Some(1));
        assert_eq!((c.state.player.block,c.state.enemies[0].hp),(2*amount,hp-face(up,5,7)),"two fully blocked hits each return Block");

        let mut zero=watcher(entry(EncounterId::OrbWalker),&[(TalkToTheHand,up),(FlyingSleeves,0)],9);
        play(&mut zero,1,Some(1));zero.state.player_full_powers.push(full_power("strength",-999));
        let hp=zero.state.enemies[0].hp;play(&mut zero,2,Some(1));
        assert_eq!((zero.state.player.block,zero.state.enemies[0].hp),(2*amount,hp),"zero-damage attack hits still call Block Return");

        let mut artifact=watcher(entry(EncounterId::OrbWalker),&[(TalkToTheHand,up)],9);
        artifact.state.enemies[0].full_powers.push(full_power("artifact",1));
        play(&mut artifact,1,Some(1));assert_eq!((power(&artifact,0,"block_return"),power(&artifact,0,"artifact")),(0,0));
    }
    // F11 disclosed project decision: normal Attack damage callbacks qualify per hit even
    // when Corrupt Heart's Invincible caps the committed HP loss; THORNS/HP_LOSS do not.
    let mut heart=watcher(heart_entry(),&[(TalkToTheHand,0),(StrikePurple,0),(StrikePurple,0)],9);
    heart.state.enemies[0].full_powers.retain(|p|p.id.0!="artifact");
    heart.state.enemies[0].full_powers.iter_mut().find(|p|p.id.0=="beat_of_death").unwrap().state.amount=0;
    play(&mut heart,1,Some(1));assert_eq!((power(&heart,0,"block_return"),heart.state.player.block),(2,0),"Talk's own Attack hit precedes installation");
    heart.state.enemies[0].full_powers.iter_mut().find(|p|p.id.0=="invincible").unwrap().state.amount=1;
    play(&mut heart,2,Some(1));assert_eq!(heart.state.player.block,2,"Invincible-capped normal hit qualifies");
    play(&mut heart,3,Some(1));assert_eq!(heart.state.player.block,4,"zero remaining Invincible allowance still qualifies");

    // Wreath of Flame: research :132. Vigor is added to each NORMAL multi-hit, then
    // consumed by that Attack's after-use callback. An Omniscience repeat sees no Vigor.
    for up in [0,1]{
        let vigor=face(up,5,8);let mut c=watcher(entry(EncounterId::OrbWalker),&[(WreathOfFlame,up),(FlyingSleeves,0)],9);
        play(&mut c,1,None);assert_eq!(player_power(&c,"vigor"),vigor);
        let hp=c.state.enemies[0].hp;play(&mut c,2,Some(1));
        assert_eq!(hp-c.state.enemies[0].hp,2*(4+vigor));assert_eq!(player_power(&c,"vigor"),0);
    }
    let mut repeated=watcher(entry(EncounterId::OrbWalker),&[(WreathOfFlame,0),(Omniscience,0),(StrikePurple,0)],9);
    place(&mut repeated,3,SelectionSourceZone::Draw);play(&mut repeated,1,None);play(&mut repeated,2,None);
    repeated.step_full(&CombatAction::SelectCard{instance:3}).unwrap();
    assert_eq!(repeated.state.enemies[0].hp,entry(EncounterId::OrbWalker).setup.enemies[0].hp-17,
        "first Strike gets Vigor 5, repeated Strike does not");

    // Wave of the Hand: research :127. Stack amounts add; only a positive gained Block
    // amount triggers Weak, and the installed power expires on the round-end hook.
    let mut wave=watcher(entry(EncounterId::ThreeDarklings),&[(WaveOfTheHand,0),(WaveOfTheHand,1)],9);
    play(&mut wave,1,None);play(&mut wave,2,None);assert_eq!(player_power(&wave,"wave_of_the_hand"),3);
    wave.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::GainBlock(0)}],None).unwrap();
    assert!((0..3).all(|slot|power(&wave,slot,"weak")==0),"zero Block does not trigger");
    wave.state.player.block=999;
    wave.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::GainBlock(1)}],None).unwrap();
    assert!((0..3).all(|slot|power(&wave,slot,"weak")==0),"a capped gain with no committed Block increase does not trigger");
    wave.state.player.block=0;
    wave.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::GainBlock(2)}],None).unwrap();
    assert!((0..3).all(|slot|power(&wave,slot,"weak")==3),"positive Block applies stacked Weak to all enemies");
    end(&mut wave);assert_eq!(player_power(&wave,"wave_of_the_hand"),0);

    // Swivel: research watcher.md:123 ("no automatic turn-end expiration. Qualifying
    // non-purge Attack use decrements a positive count"). One entitlement survives the turn
    // and the next non-purge Attack use spends it, whatever its cost or route.
    let mut swivel=watcher(entry(EncounterId::OrbWalker),&[(Swivel,0),(FlurryOfBlows,0),(StrikePurple,0),(StrikePurple,0)],9);
    play(&mut swivel,1,None);assert_eq!(swivel.state.player.block,8);assert_eq!(player_power(&swivel,"free_attack"),1);
    end(&mut swivel);assert_eq!(player_power(&swivel,"free_attack"),1,"entitlement has no turn expiry");
    // The end-turn draw leaves a legal hand-owned Attack; move the fixed identities back
    // from ordinary piles to isolate which uses consume the entitlement.
    for id in [2,3,4]{if !in_pile(&swivel.state.piles.hand,id){place(&mut swivel,id,SelectionSourceZone::Hand);}}
    let energy=swivel.state.player.energy;play(&mut swivel,2,Some(1));
    assert_eq!((swivel.state.player.energy,player_power(&swivel,"free_attack")),(energy,0),"a zero-cost non-purge Attack still spends Swivel");
    let energy=swivel.state.player.energy;play(&mut swivel,3,Some(1));assert_eq!(swivel.state.player.energy,energy-1,"no entitlement left");
    let mut fixed=watcher(entry(EncounterId::OrbWalker),&[(Swivel,0),(StrikePurple,0),(StrikePurple,0)],9);
    play(&mut fixed,1,None);
    let energy=fixed.state.player.energy;play(&mut fixed,2,Some(1));
    assert_eq!((fixed.state.player.energy,player_power(&fixed,"free_attack")),(energy,0),"a fixed-cost Attack is free and spends Swivel");
    let energy=fixed.state.player.energy;play(&mut fixed,3,Some(1));assert_eq!(fixed.state.player.energy,energy-1);
    let mut swivel_up=watcher(entry(EncounterId::OrbWalker),&[(Swivel,1)],9);play(&mut swivel_up,1,None);
    assert_eq!((swivel_up.state.player.block,player_power(&swivel_up,"free_attack")),(11,1));
    // Source law (watcher.md:123): the count is spent at the shared CardPlayed owner by any
    // non-purge Attack use, so Omniscience's first play of the original Attack spends it
    // (its purge repeat copy does not), and a created Attack played from hand spends it.
    let mut autoplay=watcher(entry(EncounterId::OrbWalker),&[(Swivel,0),(Omniscience,0),(StrikePurple,0)],9);
    place(&mut autoplay,3,SelectionSourceZone::Draw);play(&mut autoplay,1,None);play(&mut autoplay,2,None);
    autoplay.step_full(&CombatAction::SelectCard{instance:3}).unwrap();
    assert_eq!(player_power(&autoplay,"free_attack"),0,"the non-purge autoplayed original spends the entitlement");
    let mut created=watcher(entry(EncounterId::OrbWalker),&[(Swivel,0),(DefendPurple,0)],9);play(&mut created,1,None);
    let create=cards::watcher::create_action(&created.state,Smite,1).unwrap();
    created.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:create}],None).unwrap();
    let smite=piles::owned_full_cards(&created.state).find(|card|card.id==Smite).unwrap().instance.0;
    play(&mut created,smite,Some(1));assert_eq!(player_power(&created,"free_attack"),0,"created Attack played from hand consumes Swivel");

    // Simmering Fury: research :121. Stack draw amounts, enter Wrath once next turn,
    // draw before ordinary turn draw, and remove the delayed power after firing.
    let mut simmer=watcher(entry(EncounterId::OrbWalker),&[(SimmeringFury,0),(SimmeringFury,1),(DefendPurple,0),(DefendPurple,0),(DefendPurple,0),(DefendPurple,0),(DefendPurple,0),(DefendPurple,0),(DefendPurple,0)],9);
    play(&mut simmer,1,None);play(&mut simmer,2,None);assert_eq!(player_power(&simmer,"simmering_fury"),5);
    let draws=hooks(&simmer,Hook::CardDrawn);end(&mut simmer);
    assert_eq!(simmer.state.character.stance,Stance::Wrath);assert_eq!(player_power(&simmer,"simmering_fury"),0);
    assert_eq!(hooks(&simmer,Hook::CardDrawn)-draws,6,"five delayed cards plus one normal start draw");

    // Fasting: research :104. Strength/Dexterity apply before the independently Artifact-
    // preventable Energy Down. The debuff loses one energy at each later recharge.
    for up in [0,1]{
        let n=face(up,3,4);let mut blocked=watcher(entry(EncounterId::OrbWalker),&[(Fasting,up)],9);
        blocked.state.player_full_powers.push(full_power("artifact",1));play(&mut blocked,1,None);
        assert_eq!((player_power(&blocked,"strength"),player_power(&blocked,"dexterity"),player_power(&blocked,"energy_down"),player_power(&blocked,"artifact")),(n,n,0,0));
        let mut applied=watcher(entry(EncounterId::OrbWalker),&[(Fasting,up)],9);play(&mut applied,1,None);
        assert_eq!((player_power(&applied,"strength"),player_power(&applied,"dexterity"),player_power(&applied,"energy_down")),(n,n,1));
        end(&mut applied);assert_eq!(applied.state.player.energy,8,"Energy Down applies after the next base recharge");
    }

    // Deva Form: implementation note DEVA_FORM and research :485-492. Keep growth step
    // `a` in the power and next recharge `g` in its native accessor: grant g, then g+=a;
    // a later stack increases a by one and g by one.
    for up in [0,1]{
        let mut c=watcher(entry(EncounterId::OrbWalker),&[(DevaForm,up)],9);play(&mut c,1,None);
        assert_eq!((player_power(&c,"deva_form"),c.deva_form_energy_gain()),(1,1));
        end(&mut c);assert_eq!(c.state.player.energy,10);assert_eq!((player_power(&c,"deva_form"),c.deva_form_energy_gain()),(1,2));
        end(&mut c);assert_eq!(c.state.player.energy,11);assert_eq!((player_power(&c,"deva_form"),c.deva_form_energy_gain()),(1,3));
        end(&mut c);assert_eq!(c.state.player.energy,12);assert_eq!((player_power(&c,"deva_form"),c.deva_form_energy_gain()),(1,4));
    }
    let mut stacked=watcher(entry(EncounterId::OrbWalker),&[(DevaForm,0),(DevaForm,0)],9);
    play(&mut stacked,1,None);play(&mut stacked,2,None);
    assert_eq!((player_power(&stacked,"deva_form"),stacked.deva_form_energy_gain()),(2,2));
    for (energy,gain) in [(11,4),(13,6),(15,8)]{
        end(&mut stacked);assert_eq!((stacked.state.player.energy,stacked.deva_form_energy_gain()),(energy,gain));
    }
    // The later copy is upgraded so it survives as an ordinary discard instead of exhausting.
    let mut late=watcher(entry(EncounterId::OrbWalker),&[(DevaForm,0),(DevaForm,1)],9);
    play(&mut late,1,None);end(&mut late);
    assert_eq!((late.state.player.energy,player_power(&late,"deva_form"),late.deva_form_energy_gain()),(10,1,2));
    place(&mut late,2,SelectionSourceZone::Hand);
    play(&mut late,2,None);
    assert_eq!((player_power(&late,"deva_form"),late.deva_form_energy_gain()),(2,3));
    end(&mut late);assert_eq!((late.state.player.energy,late.deva_form_energy_gain()),(12,5));
    end(&mut late);assert_eq!((late.state.player.energy,late.deva_form_energy_gain()),(14,7));

    // F13 disclosed project decision (contract Binding 9): turn recharge is set
    // synchronously to max energy minus Energy Down; Deva's gain then resolves
    // before Simmering's stance/draw callback. Calm/Lotus energy, Rushdown draws,
    // and Mental Fortress Block remain queued through the shared stance owner.
    let deck=[(DevaForm,0),(SimmeringFury,0),(Rushdown,0),(MentalFortress,0),
        (DefendPurple,0),(DefendPurple,0),(DefendPurple,0),(DefendPurple,0),(DefendPurple,0),(DefendPurple,0)];
    let mut calm=watcher_with_relics(entry(EncounterId::OrbWalker),&deck,9,vec![crate::content::RelicId::VioletLotus]);
    play(&mut calm,1,None);play(&mut calm,2,None);play(&mut calm,3,None);play(&mut calm,4,None);
    calm.state.character.stance=Stance::Calm;
    for id in 5..=10{place(&mut calm,id,SelectionSourceZone::Draw);}
    let draws=hooks(&calm,Hook::CardDrawn);end(&mut calm);
    assert_eq!((calm.state.character.stance,calm.state.player.energy,calm.state.player.block,player_power(&calm,"deva_form"),calm.deva_form_energy_gain()),(Stance::Wrath,13,4,1,2));
    assert_eq!(hooks(&calm,Hook::CardDrawn)-draws,5,"Simmering 2, ordinary draw 1, then Rushdown 2");

    let mut divinity=watcher(entry(EncounterId::OrbWalker),&deck,9);
    play(&mut divinity,1,None);play(&mut divinity,2,None);play(&mut divinity,3,None);play(&mut divinity,4,None);
    divinity.state.character.stance=Stance::Divinity;
    for id in 5..=10{place(&mut divinity,id,SelectionSourceZone::Draw);}
    end(&mut divinity);
    assert_eq!((divinity.state.character.stance,divinity.state.player.energy,divinity.state.player.block),(Stance::Wrath,10,8),
        "Divinity exits to Neutral before Simmering enters Wrath; Mental Fortress reacts to each transition");

    // Collect: research :100, :417-427. Captured X installs one Miracle+ per future
    // recharge; upgraded contributes X+1. The duration power decrements once each turn.
    for (up,x) in [(0,2),(1,2)]{
        let turns=x+up as i32;let mut c=watcher(entry(EncounterId::OrbWalker),&[(Collect,up)],9);c.state.player.energy=x;
        play(&mut c,1,None);assert_eq!(player_power(&c,"collect"),turns);assert!(in_pile(&c.state.piles.exhaust,1));
        for made in 1..=turns{
            end(&mut c);assert_eq!(total_cards(&c,Miracle),made as usize);
            assert_eq!(piles::owned_full_cards(&c.state).filter(|card|card.id==Miracle).map(|card|card.upgrades).collect::<Vec<_>>(),vec![1;made as usize]);
            assert_eq!(player_power(&c,"collect"),turns-made);
        }
    }

    // Deus Ex Machina: implementation note :47-48 and research :142, :365. Actual draw
    // creates base Miracles then exhausts the owner; creation, Scry and Meditate movement
    // do not invoke that draw callback.
    for up in [0,1]{
        let count=face(up,2,3);let mut c=watcher_with_relics_opening_draw(entry(EncounterId::OrbWalker),&[(DeusExMachina,up)],9,0,vec![]);
        let before=total_cards(&c,Miracle);
        c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Draw(1)}],None).unwrap();
        assert_eq!(total_cards(&c,Miracle)-before,count as usize);assert!(in_pile(&c.state.piles.exhaust,1));
        assert_eq!(piles::owned_full_cards(&c.state).filter(|card|card.id==Miracle).map(|card|card.upgrades).collect::<Vec<_>>(),vec![0;count as usize]);
    }
    // The shared generated insertion owner adds a Deus fixture directly to hand without
    // publishing an actual-draw callback. M25 does not admit Deus as a generated-card kind.
    let mut inserted=watcher(entry(EncounterId::OrbWalker),&[(DefendPurple,0)],9);
    let mut deus=m25_watcher_live_card(DeusExMachina,CardInstanceId(2),0).unwrap();
    deus.origin=CombatCardRef::CombatLocal(2);deus.copy_family=2;deus.copy_provenance=card_state::CopyProvenance::Generated;
    piles::insert_generated(&mut inserted.state,deus,command::CardDestination::Hand).unwrap();
    assert!(in_pile(&inserted.state.piles.hand,2));assert_eq!(total_cards(&inserted,Miracle),0,"hand insertion is not a draw");

    let mut created=watcher_with_relics_opening_draw(entry(EncounterId::OrbWalker),&[(ThirdEye,0),(DeusExMachina,0)],9,0,vec![]);
    place(&mut created,1,SelectionSourceZone::Hand);play(&mut created,1,None);
    let candidates=created.public_full().unwrap().selection.unwrap().candidates;
    assert!(candidates.iter().any(|candidate|candidate.instance==CardInstanceId(2)));
    created.step_full(&CombatAction::SelectCard{instance:2}).unwrap();created.step_full(&CombatAction::ConfirmSelection).unwrap();
    assert!(in_pile(&created.state.piles.discard,2));assert_eq!(total_cards(&created,Miracle),0,"Scry discard is not a draw");
    let mut meditate=watcher_with_relics_opening_draw(entry(EncounterId::OrbWalker),&[(Meditate,0),(DeusExMachina,0)],9,0,vec![]);
    place(&mut meditate,1,SelectionSourceZone::Hand);place(&mut meditate,2,SelectionSourceZone::Discard);play(&mut meditate,1,None);
    assert!(in_pile(&meditate.state.piles.hand,2));assert_eq!(total_cards(&meditate,Miracle),0,"Meditate retrieval is not a draw");
}

#[test]
fn m25_powers_turn_control(){
    // Conclude: design §5.1/research :101, literal 12/16 all-enemy damage before one end.
    for up in [0,1]{
        let damage=face(up,12,16);let mut c=watcher(entry(EncounterId::ThreeDarklings),&[(Conclude,up)],9);
        let before=c.state.enemies.iter().map(|enemy|enemy.hp).collect::<Vec<_>>();play(&mut c,1,None);
        assert_eq!(c.state.enemies.iter().map(|enemy|enemy.hp).collect::<Vec<_>>(),before.iter().map(|hp|hp-damage).collect::<Vec<_>>());
        assert_eq!(c.state.turn,2);assert_eq!(hooks(&c,Hook::PlayerEndTurn),1);assert_eq!(hooks(&c,Hook::TurnEnded),1);
    }
    // Vault: research :153 and implementation note :103-106. The extra turn preserves the
    // player end/start hooks once each, skips EnemyTurnStarted and TurnEnded, and keeps Vault exhausted.
    let mut vault=watcher(entry(EncounterId::OrbWalker),&[(DevaForm,0),(Vault,0),(DefendPurple,0)],9);
    play(&mut vault,1,None);vault.state.player_full_powers.push(full_power("metallicize",5));
    let starts=hooks(&vault,Hook::TurnStartedPreDraw);let draws=hooks(&vault,Hook::CardDrawn);play(&mut vault,2,None);
    assert_eq!((vault.state.turn,vault.state.player.energy),(2,10));assert_eq!(vault.state.player.block,0);
    assert_eq!(hooks(&vault,Hook::PlayerEndTurn),1);assert_eq!(hooks(&vault,Hook::PlayerPreEndTurnCards),1);
    assert_eq!(hooks(&vault,Hook::TurnStartedPreDraw)-starts,1);assert_eq!(hooks(&vault,Hook::CardDrawn)-draws,1);
    assert_eq!(hooks(&vault,Hook::EnemyTurnStarted),0);assert_eq!(hooks(&vault,Hook::TurnEnded),0);
    assert!(in_pile(&vault.state.piles.exhaust,2));
    let mut upgraded=watcher(entry(EncounterId::OrbWalker),&[(Vault,1)],2);play(&mut upgraded,1,None);
    assert_eq!((upgraded.state.turn,upgraded.state.player.energy,hooks(&upgraded,Hook::PlayerEndTurn)),(2,2,1),
        "upgraded Vault's two-energy cost permits the extra turn from two energy");

    // Blasphemy: implementation note :23-26 and research :139, :539-542. The positive
    // Blasphemer buff survives a stance change and queues 99,999 HP_LOSS at next start.
    for up in [0,1]{
        let mut c=watcher(entry(EncounterId::OrbWalker),&[(Blasphemy,up)],9);
        c.state.player_full_powers.push(full_power("artifact",1));c.state.character.stance=Stance::Calm;
        set_intent(&mut c,0,"orb_claw");play(&mut c,1,None);
        assert_eq!((c.state.character.stance,player_power(&c,"blasphemer"),player_power(&c,"artifact")),(Stance::Divinity,1,1));
        c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Character(CharacterEffect::Stance(Stance::Wrath))}],None).unwrap();
        assert_eq!(player_power(&c,"blasphemer"),1,"stance transition does not remove Blasphemer");
        end(&mut c);assert_eq!(c.state.outcome,Some(Outcome::Death));assert_eq!(c.state.player.hp,0);
        assert_eq!(player_power(&c,"artifact"),1,"player Artifact does not block positive Blasphemer");
        assert!(c.state.hook_events.iter().any(|event|event.hook==Hook::Damage));
    }
}

#[test]
fn m25_powers_stacked_turn_endings(){
    // User ruling, SHA-256 4dfdfb39f2f6b93ef600f2d344bf6ade9b1ee2ad6171cfe5e6cb7f68a40e96a7:
    // Vault as Time Eater's 12th card keeps Time Warp's reset and queued +2 Strength for
    // every actor, then Vault governs the single end and extra turn. The ruling explicitly
    // skips enemy action/end-round; we disclose F2 Invincible below.
    let mut vault=watcher(time_eater(11,"time_reverb"),&[(Vault,0),(DefendPurple,0)],3);
    vault.state.player.block=0;let draws=hooks(&vault,Hook::CardDrawn);let starts=hooks(&vault,Hook::TurnStartedPreDraw);
    let hp=vault.state.player.hp;play(&mut vault,1,None);
    assert_eq!((vault.state.turn,power(&vault,0,"time_warp"),power(&vault,0,"strength")),(2,0,2),"Time Warp callback resets and queues Strength before Vault takes control");
    assert_eq!(player_power(&vault,"strength"),0,"Time Warp's shared producer targets encounter enemies, not the player");
    assert_eq!((vault.state.player.hp,vault.state.player.block),(hp,0),"Time Eater took no enemy turn");
    assert_eq!(hooks(&vault,Hook::PlayerEndTurn),1);assert_eq!(hooks(&vault,Hook::TurnEnded),0);
    assert_eq!(hooks(&vault,Hook::EnemyTurnStarted),0);assert_eq!(hooks(&vault,Hook::TurnStartedPreDraw)-starts,1);
    assert_eq!(hooks(&vault,Hook::CardDrawn)-draws,1,"one extra-turn start draw");assert_eq!(vault.state.player.energy,3,"one extra-turn energy refill");
    assert!(!vault.extra_turn_pending());
    assert!(in_pile(&vault.state.piles.exhaust,1));assert_eq!(vault.state.enemies[0].block,0);
    assert!(vault.state.player_full_powers.iter().all(|power|!matches!(power.id.0,"weak"|"vulnerable"|"frail")));
    // Disclosed F2 project decision: Invincible resets only at MonsterStart, so the skipped
    // round leaves its current allowance unchanged. The first extra turn does not reset it.
    let mut heart=watcher(heart_entry(),&[(Vault,0)],3);let invincible=power(&heart,0,"invincible");
    play(&mut heart,1,None);assert_eq!(power(&heart,0,"invincible"),invincible,"Vault skips MonsterStart, the sole Invincible reset owner");

    // Conclude as the 12th card also ends once, but its normal request lets Time Eater take
    // one turn and one end-round callback. Its attack damage resolves before that request.
    for up in [0,1]{
        let damage=face(up,12,16);let mut c=watcher(time_eater(11,"time_ripple"),&[(Conclude,up),(DefendPurple,0)],3);
        let hp=c.state.enemies[0].hp;play(&mut c,1,None);
        assert_eq!((c.state.turn,power(&c,0,"time_warp"),power(&c,0,"strength")),(2,0,2));
        assert_eq!(c.state.enemies[0].hp,hp-damage);assert_eq!(hooks(&c,Hook::PlayerEndTurn),1);
        assert_eq!(hooks(&c,Hook::EnemyTurnStarted),1);assert_eq!(hooks(&c,Hook::TurnEnded),1);
    }

    // Review Focus 5: Omniscience selecting Vault ends after its first child play and
    // cancels the queued repeat. The source M25 Vault note says a doubled Vault can end
    // the turn before its second play (implementation-notes.json:103-106).
    let mut omni=watcher(entry(EncounterId::OrbWalker),&[(Omniscience,0),(Vault,0),(DefendPurple,0)],9);
    place(&mut omni,2,SelectionSourceZone::Draw);play(&mut omni,1,None);
    omni.step_full(&CombatAction::SelectCard{instance:2}).unwrap();
    assert_eq!(omni.state.turn,2);assert_eq!(hooks(&omni,Hook::PlayerEndTurn),1);assert_eq!(hooks(&omni,Hook::TurnEnded),0);
    assert_eq!(omni.state.hook_events.iter().filter(|event|event.hook==Hook::CardPlayed&&event.source==Source::Card(handle(2))).count(),1);
    assert!(in_pile(&omni.state.piles.exhaust,2));

    // Blasphemer still fires on Vault's extra-turn start and can end combat there.
    let mut death=watcher(entry(EncounterId::OrbWalker),&[(Blasphemy,0),(Vault,0)],9);
    death.state.player_full_powers.push(full_power("collect",1));
    play(&mut death,1,None);play(&mut death,2,None);
    assert_eq!((death.state.turn,death.state.player.hp,death.state.outcome),(2,0,Some(Outcome::Death)));
    assert_eq!(total_cards(&death,Miracle),0,"F3 disclosed project decision: Blasphemer HP_LOSS resolves before Watcher start-of-turn Collect creation, so lethal terminal state suppresses later callbacks");
    assert!(in_pile(&death.state.piles.exhaust,2));
}
