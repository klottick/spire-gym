use spire_core::{
    combat::{self, CombatAction, SelectionOperation},
    content::{CardId, Character, EncounterId, RelicId},
    items::CombatItemContext,
    run::{CreationProfile, GameAction, Run, RunConfig, RunRng, RoomScenario},
};
use spire_env::{
    encode_run, encode_run_m10, m10_run_observation_spec, run_observation_spec,
    RunCapacities, RunSessionError, RUN_OBSERVATION_SCHEMA_ID,
};

fn initial_m10_run() -> Run {
    let mut profile = CreationProfile::experienced();
    profile.neow_eligible = false;
    Run::new_room_scenario(
        RunConfig { character: Character::Ironclad, ascension: 0 },
        profile,
        RunRng::new([91; 32], combat::rng::Partition::Test, 0),
        RoomScenario::M10IroncladHeart,
    ).unwrap()
}

fn m10_decision() -> spire_core::run::PublicDecision {
    initial_m10_run().decision().unwrap()
}

fn align_public_deck(decision: &mut spire_core::run::PublicDecision, cards: &[combat::LiveCard]) {
    decision.objects = cards.iter().map(|card| spire_core::run::PublicObject { misc:0,
        instance: card.instance.0,
        id: card.id,
        upgrades: card.upgrades,
    }).collect();
}

fn native_combat_candidates(actions: Vec<CombatAction>, first_id: u64) -> Vec<spire_core::run::Candidate> {
    actions.into_iter().enumerate().map(|(index, action)| spire_core::run::Candidate {
        id: spire_core::run::ChoiceId(first_id + index as u64),
        action: GameAction::Combat(action),
        label: "native legal combat action",
        neow_offer: None,
        object: None,
    }).collect()
}

fn m10_map_boundary() -> (Run,spire_core::run::PublicDecision,Option<spire_core::run::map::PublicMap>) {
    let mut run=initial_m10_run();
    let first=run.decision().unwrap().candidates[0].action.clone();
    run.step(&first).unwrap();
    let decision=run.decision().unwrap();
    let map=run.public_map();
    (run,decision,map)
}

fn m10_combat_decision(encounter: EncounterId) -> spire_core::run::PublicDecision {
    let mut decision = m10_decision();
    let cards = (1..=10).map(|id| combat::permanent_live_card(CardId::Strike, combat::CardInstanceId(id), 0)).collect::<Vec<_>>();
    align_public_deck(&mut decision, &cards);
    // Reuse the existing native Shield/Spear characterization seed rather than
    // rewriting the intents after the engine publishes them.
    let setup_seed = if encounter == EncounterId::ShieldAndSpear { [19; 32] } else { [92; 32] };
    let mut setup_rng = RunRng::new(setup_seed, combat::rng::Partition::Test, 0);
    let setup = combat::enemies::setup(
        encounter,
        4,
        0,
        &mut setup_rng,
        combat::enemies::SetupContext {
            player: combat::PlayerState { hp: 80, max_hp: 80, block: 0, energy: 3, max_energy: 3, draw_per_turn: 5, powers: vec![] },
            player_powers: vec![],
            character: Character::Ironclad,
            cards,
            opening_draw: 5,
            encounter_visit: 0,
        },
    ).unwrap();
    let mut items = Some(CombatItemContext::new(vec![RelicId::BurningBlood], vec![None, None, None]));
    let native = combat::Combat::enter_with_item_context(setup, &mut items).unwrap();
    decision.combat = Some(native.public_full().unwrap());
    decision.room = None;
    decision.candidates.clear();
    decision.offers.clear();
    decision.selection = None;
    decision
}

fn m10_headbutt_draw_decision() -> spire_core::run::PublicDecision {
    let mut decision = m10_decision();
    let cards = std::iter::once(combat::permanent_live_card(CardId::Headbutt, combat::CardInstanceId(1), 0))
        .chain((2..=6).map(|instance| combat::permanent_live_card(CardId::Strike, combat::CardInstanceId(instance), 0)))
        .collect::<Vec<_>>();
    align_public_deck(&mut decision, &cards);
    let setup = combat::enemies::setup(
        EncounterId::CorruptHeart,
        4,
        0,
        &mut RunRng::new([98; 32], combat::rng::Partition::Test, 0),
        combat::enemies::SetupContext {
            player: combat::PlayerState { hp: 80, max_hp: 80, block: 0, energy: 3, max_energy: 3, draw_per_turn: 5, powers: vec![] },
            player_powers: vec![],
            character: Character::Ironclad,
            cards,
            opening_draw: 6,
            encounter_visit: 0,
        },
    ).unwrap();
    let mut items = Some(CombatItemContext::new(vec![RelicId::BurningBlood], vec![None, None, None]));
    let mut native = combat::Combat::enter_with_item_context(setup, &mut items).unwrap();
    let hand = native.public_full().unwrap().hand;
    let headbutt = hand.iter().find(|card| card.id == CardId::Headbutt).unwrap().instance.0;
    let strike = hand.iter().find(|card| card.id == CardId::Strike).unwrap().instance.0;
    native.step_full(&CombatAction::PlayCard { instance: strike, target: Some(combat::TargetId(1)) }).unwrap();
    native.step_full(&CombatAction::PlayCard { instance: headbutt, target: Some(combat::TargetId(1)) }).unwrap();
    let public = native.public_full().unwrap();
    assert_eq!(public.draw_count, 1);
    assert_eq!(public.known_draw.len(), 2);
    assert_eq!((public.known_draw[0].from_top, public.known_draw[0].offset), (true, 0));
    assert_eq!((public.known_draw[1].from_top, public.known_draw[1].offset), (false, 0));
    assert_eq!(public.known_draw[0].card.instance, public.known_draw[1].card.instance);
    decision.combat = Some(public);
    decision.room = None;
    decision.candidates.clear();
    decision.offers.clear();
    decision.selection = None;
    decision
}

fn m10_dual_wield_selection_decision() -> spire_core::run::PublicDecision {
    let mut decision = m10_decision();
    let mut setup_rng = RunRng::new([93; 32], combat::rng::Partition::Test, 0);
    let cards = std::iter::once(combat::permanent_live_card(CardId::DualWield, combat::CardInstanceId(1), 0))
        .chain((2..=5).map(|instance| combat::permanent_live_card(CardId::Strike, combat::CardInstanceId(instance), 0)))
        .collect::<Vec<_>>();
    align_public_deck(&mut decision, &cards);
    let setup = combat::enemies::setup(EncounterId::CorruptHeart, 4, 0, &mut setup_rng,
        combat::enemies::SetupContext {
            player: combat::PlayerState { hp: 80, max_hp: 80, block: 0, energy: 3, max_energy: 3, draw_per_turn: 5, powers: vec![] },
            player_powers: vec![], character: Character::Ironclad, cards, opening_draw: 5, encounter_visit: 0,
        }).unwrap();
    let mut items = Some(CombatItemContext::new(vec![RelicId::BurningBlood], vec![None, None, None]));
    let mut native = combat::Combat::enter_with_item_context(setup, &mut items).unwrap();
    let parent = native.public_full().unwrap().hand.iter().find(|card| card.id == CardId::DualWield).unwrap().instance.0;
    native.step_full(&CombatAction::PlayCard { instance: parent, target: None }).unwrap();
    let legal = native.legal_actions_full().unwrap();
    assert!(!legal.contains(&CombatAction::ConfirmSelection), "fixed-size Dual Wield has no native confirmation action");
    let public = native.public_full().unwrap();
    let selection = public.selection.as_ref().unwrap();
    assert_eq!(selection.operation, SelectionOperation::CopyToHand { copies: 2 });
    assert_eq!((selection.minimum, selection.maximum), (1, 1));
    decision.combat = Some(public);
    decision.room = None;
    decision.offers.clear();
    decision.selection = None;
    decision.candidates = native_combat_candidates(legal, 1000);
    decision
}

fn m10_elixir_selection_decision() -> spire_core::run::PublicDecision {
    let mut decision = m10_decision();
    let cards = (1..=5).map(|instance| combat::permanent_live_card(CardId::Strike, combat::CardInstanceId(instance), 0)).collect::<Vec<_>>();
    align_public_deck(&mut decision, &cards);
    let setup = combat::enemies::setup(EncounterId::CorruptHeart, 4, 0, &mut RunRng::new([94; 32], combat::rng::Partition::Test, 0),
        combat::enemies::SetupContext {
            player: combat::PlayerState { hp: 80, max_hp: 80, block: 0, energy: 3, max_energy: 3, draw_per_turn: 5, powers: vec![] },
            player_powers: vec![], character: Character::Ironclad, cards, opening_draw: 5, encounter_visit: 0,
        }).unwrap();
    let mut items = Some(CombatItemContext::new(vec![RelicId::BurningBlood], vec![Some(spire_core::content::PotionId::Elixir), None, None]));
    let mut native = combat::Combat::enter_with_item_context(setup, &mut items).unwrap();
    native.step_full(&CombatAction::UsePotion { slot: 0, target: None }).unwrap();
    let legal = native.legal_actions_full().unwrap();
    assert!(legal.contains(&CombatAction::ConfirmSelection), "the optional native Elixir selector offers confirmation");
    let public = native.public_full().unwrap();
    let selection = public.selection.as_ref().expect("Elixir opens its public card selector");
    assert_eq!(selection.minimum, 0);
    assert!(selection.maximum > selection.minimum);
    decision.combat = Some(public);
    decision.room = None;
    decision.offers.clear();
    decision.selection = None;
    decision.candidates = native_combat_candidates(legal, 1200);
    decision
}

fn m10_demon_form_decision() -> spire_core::run::PublicDecision {
    let mut decision=m10_decision();
    let cards=std::iter::once(combat::permanent_live_card(CardId::DemonForm,combat::CardInstanceId(1),0))
        .chain((2..=5).map(|instance|combat::permanent_live_card(CardId::Strike,combat::CardInstanceId(instance),0))).collect::<Vec<_>>();
    align_public_deck(&mut decision,&cards);
    let setup=combat::enemies::setup(EncounterId::CorruptHeart,4,0,&mut RunRng::new([95;32],combat::rng::Partition::Test,0),combat::enemies::SetupContext{
        player:combat::PlayerState{hp:80,max_hp:80,block:0,energy:3,max_energy:3,draw_per_turn:5,powers:vec![]},player_powers:vec![],character:Character::Ironclad,
        cards,opening_draw:5,encounter_visit:0}).unwrap();
    let mut items=Some(CombatItemContext::new(vec![RelicId::BurningBlood],vec![None,None,None]));
    let mut native=combat::Combat::enter_with_item_context(setup,&mut items).unwrap();
    let parent=native.public_full().unwrap().hand.iter().find(|card|card.id==CardId::DemonForm).unwrap().instance.0;
    native.step_full(&CombatAction::PlayCard{instance:parent,target:None}).unwrap();
    let public=native.public_full().unwrap();
    assert!(public.player.powers.iter().any(|power|power.id.0=="demon_form"&&power.amount==2),"the actual Demon Form action publishes its power");
    decision.combat=Some(public);
    decision.room=None;
    decision.offers.clear();
    decision.candidates.clear();
    decision.selection=None;
    decision
}

fn group<'a>(observation: &'a spire_env::RunObservation, key: &str) -> &'a spire_env::RunFeatureGroup {
    observation.group(key).unwrap()
}

#[test]
fn m10_has_a_separate_opt_in_v2_schema_and_leaves_m9_v1_frozen() {
    let caps = RunCapacities::m10_default();
    assert_eq!(caps, RunCapacities::m9_default());
    let v1 = run_observation_spec(&RunCapacities::m9_default()).unwrap();
    assert_eq!(v1.fingerprint(), run_observation_spec(&RunCapacities::m9_default()).unwrap().fingerprint());
    assert_eq!(v1.fingerprint().iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
        "0893bce86e4b3abf11aa384c501503a91ae37f688ec605ab40b814ad365cb999");
    let spec = m10_run_observation_spec(&caps).unwrap();
    let descriptor = spec.descriptor();
    assert_eq!(descriptor["schema_id"], "spire-gym-run-observation-v2");
    assert_ne!(descriptor["schema_id"], RUN_OBSERVATION_SCHEMA_ID);
    assert_eq!(descriptor["profile_id"], "M10IroncladHeart");
    assert_eq!(descriptor["capacities"], v1.descriptor()["capacities"]);
    let tensors = descriptor["tensors"].as_array().unwrap();
    let columns = |key: &str| -> Vec<&str> {
        tensors.iter().find(|tensor| tensor["key"] == key).unwrap()["fields"]
            .as_array().unwrap().iter().map(|value| value.as_str().unwrap()).collect()
    };
    assert_eq!(columns("combat_cards"), [
        "zone", "zone_position", "card_kind", "upgrades", "effective_cost", "permanent_row", "stasis_enemy_row", "selected",
        "base_damage", "base_block", "magic_number", "damage_bonus", "cost_kind", "free_play", "exhaust_on_finish", "ethereal", "innate", "temporary_cost",
    ]);
    assert_eq!(columns("enemies").last(), Some(&"intent_hits"));
    assert_eq!(columns("combat_state"), ["facing", "surrounded", "heart_damage_remaining", "heart_cap"]);
    assert_eq!(columns("selection_counts"), ["required", "selected_count", "minimum", "maximum", "confirm_available", "operation"]);
    assert_eq!(columns("known_draw"), ["from_top", "offset", "combat_card_row"]);
    assert_eq!(tensors.iter().find(|tensor| tensor["key"] == "known_draw").unwrap()["shape"], serde_json::json!([caps.combat_cards, 3]));
    for table in descriptor["vocabularies"].as_object().unwrap().values() {
        let entries = table.as_array().unwrap();
        for (i, entry) in entries.iter().enumerate() {
            assert!(!entries[..i].contains(entry), "duplicate v2 vocabulary value {entry}");
        }
    }
    let appended = |key: &str, old: &serde_json::Value| -> Vec<String> {
        let old_len=old[key].as_array().unwrap().len();
        descriptor["vocabularies"][key].as_array().unwrap()[old_len..].iter().map(|value|value.as_str().unwrap().to_owned()).collect()
    };
    let v1_descriptor=v1.descriptor();
    let old_vocab=&v1_descriptor["vocabularies"];
    assert_eq!(appended("card",old_vocab), [
        "armaments","searing_blow","body_slam","carnage","clash","hemokinesis","pummel","sword_boomerang","uppercut","wild_strike","immolate","feed","reaper","infernal_blade",
        "flex","limit_break","blood_for_blood","rampage","dropkick","entrench","ghostly_armor","intimidate","power_through","reckless_charge","seeing_red","shockwave","spot_weakness","true_grit","headbutt","warcry","battle_trance","exhume","dual_wield","havoc","double_tap","whirlwind",
        "evolve","fire_breathing","flame_barrier","juggernaut","rage","rupture","corruption","dark_embrace","feel_no_pain","sentinel","second_wind","sever_soul","fiend_fire","barricade","berserk","brutality","demon_form","metallicize","combust",
    ]);
    assert_eq!(appended("enemy",old_vocab), ["spire_shield","spire_spear","corrupt_heart"]);
    assert_eq!(appended("encounter",old_vocab), ["shield_and_spear","corrupt_heart"]);
    assert_eq!(appended("relic",old_vocab), ["red_skull","paper_phrog","champions_belt","charons_ashes","self_forming_clay","orichalcum","brimstone","mark_of_pain","runic_cube"]);
    assert_eq!(appended("potion",old_vocab), ["elixir","heart_of_iron"]);
    assert_eq!(appended("profile",old_vocab), ["M10IroncladHeart"]);
    assert_eq!(appended("room",old_vocab), ["heart_victory","true_victory"]);
    assert_eq!(appended("verb",old_vocab), ["confirm_selection"]);
    assert_eq!(appended("zone",old_vocab), ["known_draw_top","known_draw_bottom","playing"]);
    assert_eq!(appended("row_group",old_vocab), ["combat_state","known_draw"]);
    assert_eq!(appended("power",old_vocab), ["berserk","brutality","combust","corruption","dark_embrace","demon_form","double_tap","evolve","feel_no_pain","fire_breathing","flame_barrier","juggernaut","no_draw","rage","rupture",
        "beat_of_death","invincible","surrounded","back_attack"]);
    assert_eq!(appended("intent",old_vocab), ["shield_bash","shield_fortify","shield_smash","spear_burn_strike","spear_skewer","spear_strength","heart_debilitate","heart_blood_shots","heart_echo","heart_buff"]);
    assert_eq!(descriptor["vocabularies"]["card_cost_kind"], serde_json::json!(["padding","fixed","x","unplayable"]));
    assert_eq!(descriptor["vocabularies"]["selection_operation"], serde_json::json!(["padding","move_hand","move_discard","move_exhaust","move_draw_top","upgrade","topdeck","return_from_exhaust","copy_to_hand_1","copy_to_hand_2","copy_to_hand_3","select_permanent"]));
}

#[test]
fn actual_m10_public_boundary_uses_v2_while_v1_explicitly_rejects_it() {
    let run = initial_m10_run();
    let decision = run.decision().unwrap();
    let map = run.public_map();
    assert_eq!(encode_run(&decision, map.as_ref(), &RunCapacities::m9_default()),
        Err(RunSessionError::Input("M10 profile is not representable in the M9 v1 observation schema")));
    let observation = encode_run_m10(&decision, map.as_ref(), &RunCapacities::m10_default()).unwrap();
    assert_eq!(group(&observation, "context").values[17], 7, "M10 is an appended explicit profile ID");
    assert_eq!(observation.action_mask.iter().filter(|value| **value == 1).count(), decision.candidates.len());

    let mut profile=spire_core::run::CreationProfile::experienced();profile.neow_eligible=false;
    let m9=Run::new_room_scenario(RunConfig{character:Character::Ironclad,ascension:0},profile,
        RunRng::new([90;32],combat::rng::Partition::Test,0),RoomScenario::M9IroncladAct3).unwrap();
    assert_eq!(encode_run_m10(&m9.decision().unwrap(),None,&RunCapacities::m10_default()),
        Err(RunSessionError::Input("M10 v2 encoder requires the M10 Ironclad profile")));
}

#[test]
fn m10_rejects_unknown_typed_identity_instead_of_hashing_or_aliasing_it() {
    let mut decision=m10_combat_decision(EncounterId::CorruptHeart);
    // Deliberately invalid supplied-transport identity, not an actual native card mutation.
    decision.combat.as_mut().unwrap().hand[0].id=CardId::Unknown("strike");
    assert!(matches!(encode_run_m10(&decision,None,&RunCapacities::m10_default()),Err(RunSessionError::Input("unsupported CardId"))));
}

#[test]
fn m10_encoder_maps_a_power_created_by_an_actual_ironclad_card() {
    let decision=m10_demon_form_decision();
    let observation=encode_run_m10(&decision,None,&RunCapacities::m10_default())
        .expect("the M10 vocabulary must include a real card-produced Demon Form power");
    let descriptor=m10_run_observation_spec(&RunCapacities::m10_default()).unwrap().descriptor();
    let vocabulary=descriptor["vocabularies"]["power"].as_array().unwrap();
    let power_id=vocabulary.iter().position(|value|value=="demon_form").unwrap() as i64;
    let powers=group(&observation,"powers");
    let row=powers.values.chunks(4).position(|values|values[0]==1&&values[2]==power_id).unwrap();
    assert_eq!(powers.values[row*4+3],2);
    assert_eq!(&powers.known[row*4..row*4+4],&[1,1,1,1]);
}

#[test]
fn m10_fixed_a0_is_known_without_guessing_ascension_for_other_profiles() {
    let m10 = encode_run_m10(&m10_decision(), None, &RunCapacities::m10_default()).unwrap();
    let context = group(&m10, "context");
    assert_eq!(context.values[1], 0);
    assert_eq!(context.known[1], 1, "the actual M10 Run profile is fixed Ironclad A0");

    let mut profile = CreationProfile::experienced();
    profile.neow_eligible = false;
    let unrelated = Run::new_room_scenario(
        RunConfig { character: Character::Ironclad, ascension: 0 },
        profile,
        RunRng::new([96; 32], combat::rng::Partition::Test, 0),
        RoomScenario::M8Act3Prefix,
    ).unwrap();
    let unrelated = encode_run(&unrelated.decision().unwrap(), None, &RunCapacities::m9_default()).unwrap();
    assert_eq!(group(&unrelated, "context").known[1], 0, "unrelated profiles do not imply a known A0");
}

#[test]
fn m10_v2_names_heart_rooms_and_encodes_each_run_capacity_overflow_axis() {
    let mut decision=m10_decision();
    for (kind,room_id,terminal) in [(spire_core::run::RoomViewKind::HeartVictory,22,0),(spire_core::run::RoomViewKind::TrueVictory,23,4)] {
        decision.room=Some(spire_core::run::RoomView{kind,choices:vec![]});decision.terminal=if terminal==0{None}else{Some(spire_core::run::RunTerminal::HeartVictory)};
        decision.candidates.clear();
        let observation=encode_run_m10(&decision,None,&RunCapacities::m10_default()).unwrap();
        assert_eq!(group(&observation,"context").values[4],room_id);
        assert_eq!(group(&observation,"context").known[4],1);
        assert_eq!(group(&observation,"context").values[6],terminal);
    }

    macro_rules! overflows {
        ($($field:ident),+ $(,)?) => {$({
            let mut caps=RunCapacities::m10_default();caps.$field=usize::MAX;
            assert!(m10_run_observation_spec(&caps).is_err(),"{} dimension must not overflow shape arithmetic",stringify!($field));
        })+};
    }
    overflows!(map_nodes,map_edges,permanent_cards,combat_cards,enemies,powers,relics,potions,offers,selection,orbs,actions);
    let mut zero=RunCapacities::m10_default();zero.orbs=0;
    assert!(m10_run_observation_spec(&zero).is_err(),"reserved orb axis remains a positive fixed capacity even when Ironclad has no orb rows");
}

#[test]
fn m10_encoder_reports_actual_row_capacity_overflows_without_truncation() {
    let run=initial_m10_run();let initial=run.decision().unwrap();
    let mut small=RunCapacities::m10_default();small.permanent_cards=1;
    assert!(matches!(encode_run_m10(&initial,None,&small),Err(RunSessionError::Capacity{group:"permanent_cards",required,capacity:1}) if required>1));
    small=RunCapacities::m10_default();small.actions=1;
    assert!(matches!(encode_run_m10(&initial,None,&small),Err(RunSessionError::Capacity{group:"actions",required,capacity:1}) if required>1));
    small=RunCapacities::m10_default();small.offers=1;
    assert!(matches!(encode_run_m10(&initial,None,&small),Err(RunSessionError::Capacity{group:"offers",required,capacity:1}) if required>1));

    let (_run,decision,map)=m10_map_boundary();let public_map=map.as_ref().unwrap();
    small=RunCapacities::m10_default();small.map_nodes=1;
    assert!(matches!(encode_run_m10(&decision,map.as_ref(),&small),Err(RunSessionError::Capacity{group:"map_nodes",required,capacity:1}) if required>1));
    small=RunCapacities::m10_default();small.map_edges=1;
    assert!(matches!(encode_run_m10(&decision,map.as_ref(),&small),Err(RunSessionError::Capacity{group:"map_edges",required,capacity:1}) if required>1));
    assert!(!public_map.nodes.is_empty());

    let mut selection=initial.clone();selection.selection=Some(spire_core::run::PublicSelection{required:2,selected_count:0,eligible:selection.objects[..2].to_vec()});
    small=RunCapacities::m10_default();small.selection=1;
    assert!(matches!(encode_run_m10(&selection,None,&small),Err(RunSessionError::Capacity{group:"selection",required,capacity:1}) if required>1));

    let mut pair=m10_combat_decision(EncounterId::ShieldAndSpear);
    for (group,mut caps) in [
        ("combat_cards",RunCapacities::m10_default()),("enemies",RunCapacities::m10_default()),
        ("powers",RunCapacities::m10_default()),("relics",RunCapacities::m10_default()),
        ("potions",RunCapacities::m10_default()),
    ] {
        match group {"combat_cards"=>caps.combat_cards=1,"enemies"=>caps.enemies=1,"powers"=>caps.powers=1,
            "relics"=>{caps.relics=1;pair.combat.as_mut().unwrap().inventory.as_mut().unwrap().relics.push(RelicId::RedSkull)},
            "potions"=>caps.potions=2,_=>unreachable!()}
        let actual=encode_run_m10(&pair,None,&caps);
        assert!(matches!(actual,Err(RunSessionError::Capacity{group:actual,required,capacity}) if actual==group&&required>capacity),"missing {group} capacity rejection");
    }
}

#[test]
fn m10_encoder_keeps_card_optional_masks_and_known_draw_row_references_public() {
    let mut decision = m10_combat_decision(EncounterId::CorruptHeart);
    let combat = decision.combat.as_mut().unwrap();
    let known = combat.hand.remove(0);
    combat.known_draw.push(combat::KnownDrawCardView { card: known.clone(), from_top: true, offset: 0 });
    combat.draw_count += 1;
    combat.hand[0].values = None;

    let observation = encode_run_m10(&decision, None, &RunCapacities::m10_default()).unwrap();
    let schema = m10_run_observation_spec(&RunCapacities::m10_default()).unwrap().descriptor();
    let card_fields = schema["tensors"].as_array().unwrap().iter().find(|tensor| tensor["key"] == "combat_cards").unwrap()["fields"].as_array().unwrap();
    let field = |name: &str| card_fields.iter().position(|value| value == name).unwrap();
    let cards = group(&observation, "combat_cards");
    let width = cards.shape[1];
    let known_card_row = cards.values.chunks(width).position(|row| row[field("zone")] == 7 && row[field("zone_position")] == 0).unwrap();
    assert_eq!(cards.values[known_card_row * width + field("base_damage")], 6);
    assert_eq!(cards.known[known_card_row * width + field("base_damage")], 1);
    assert_eq!(cards.known[known_card_row * width + field("base_block")], 0, "N/A is not a known zero");
    assert_eq!(&cards.known[known_card_row * width + 8..known_card_row * width + width], &[1,0,0,0,1,1,1,1,1,1]);
    assert_eq!(&cards.values[known_card_row * width + 8..known_card_row * width + width], &[6,0,0,0,1,0,0,0,0,0]);
    let unknown_row = cards.values.chunks(width).position(|row| row[field("zone")] == 1 && row[field("zone_position")] == 0).unwrap();
    assert!(cards.known[unknown_row * width + 8..unknown_row * width + width].iter().all(|value| *value == 0));

    let known_draw = group(&observation, "known_draw");
    assert_eq!(known_draw.values[..3], [1, 0, known_card_row as i64]);
    assert_eq!(known_draw.known[..3], [1, 1, 1]);
    assert_eq!(known_draw.occupied.as_ref().unwrap()[0], 1);
    assert_eq!(known_draw.occupied.as_ref().unwrap()[1], 0, "unknown draw rows do not receive transport rows");
}

#[test]
fn m10_encoder_reuses_one_combat_row_for_native_headbutt_top_bottom_aliases() {
    let decision = m10_headbutt_draw_decision();
    let observation = encode_run_m10(&decision, None, &RunCapacities::m10_default()).unwrap();
    let cards = group(&observation, "combat_cards");
    let width = cards.shape[1];
    let draw_rows = cards.values.chunks(width).enumerate()
        .filter(|(_, row)| row[0] == 7 || row[0] == 8)
        .collect::<Vec<_>>();
    assert_eq!(draw_rows.len(), 1, "one native card identity must occupy one combat-card row");

    let known_draw = group(&observation, "known_draw");
    assert_eq!(known_draw.values[..6], [1, 0, draw_rows[0].0 as i64, 0, 0, draw_rows[0].0 as i64]);
    assert_eq!(known_draw.known[..6], [1, 1, 1, 1, 1, 1]);
    assert_eq!(known_draw.occupied.as_ref().unwrap()[..2], [1, 1], "both native endpoint facts remain represented");
}

#[test]
fn m10_synthetic_consistent_known_draw_alias_overflow_returns_typed_capacity() {
    // This is a synthetic transport-boundary case, not a native mechanics fixture.
    let mut decision = m10_combat_decision(EncounterId::CorruptHeart);
    let combat = decision.combat.as_mut().unwrap();
    let known = combat.hand[0].clone();
    combat.hand.clear();
    combat.discard.clear();
    combat.exhaust.clear();
    combat.played_powers.clear();
    combat.stasis.clear();
    combat.draw_count = 1;
    combat.known_draw = vec![
        combat::KnownDrawCardView { card: known.clone(), from_top: true, offset: 0 },
        combat::KnownDrawCardView { card: known, from_top: false, offset: 0 },
    ];

    let mut capacities = RunCapacities::m10_default();
    capacities.combat_cards = 1;
    assert_eq!(
        encode_run_m10(&decision, None, &capacities),
        Err(RunSessionError::Capacity { group: "known_draw", required: 2, capacity: 1 }),
    );
}

#[test]
fn m10_encoder_rejects_known_draw_identities_also_present_in_public_piles() {
    // Synthetic malformed transport views preserve the pre-existing zone-collision guard.
    for in_discard in [false, true] {
        let mut decision = m10_combat_decision(EncounterId::CorruptHeart);
        let combat = decision.combat.as_mut().unwrap();
        let card = combat.hand.pop().unwrap();
        if in_discard {
            combat.discard.push(card.clone());
        } else {
            combat.hand.push(card.clone());
        }
        combat.known_draw = vec![combat::KnownDrawCardView { card, from_top: true, offset: 0 }];
        assert_eq!(
            encode_run_m10(&decision, None, &RunCapacities::m10_default()),
            Err(RunSessionError::Input("known-draw card is also present in another public zone")),
        );
    }
}

#[test]
fn m10_encoder_rejects_known_draw_identity_also_present_in_playing_zone() {
    // Synthetic malformed transport view: playing cards are encoded after known-draw facts.
    let mut decision = m10_combat_decision(EncounterId::CorruptHeart);
    let combat = decision.combat.as_mut().unwrap();
    let card = combat.hand.pop().unwrap();
    combat.hand.clear();
    combat.discard.clear();
    combat.exhaust.clear();
    combat.played_powers.clear();
    combat.stasis.clear();
    combat.playing_cards = vec![card.clone()];
    combat.draw_count = 1;
    combat.known_draw = vec![combat::KnownDrawCardView { card, from_top: true, offset: 0 }];
    assert_eq!(
        encode_run_m10(&decision, None, &RunCapacities::m10_default()),
        Err(RunSessionError::Input("known-draw card is also present in another public zone")),
    );
}

#[test]
fn m10_encoder_rejects_one_known_draw_identity_at_inconsistent_positions() {
    // Synthetic malformed public facts: with two draw cards, top offset 0 and
    // bottom offset 0 identify different physical positions.
    let mut decision = m10_combat_decision(EncounterId::CorruptHeart);
    let combat = decision.combat.as_mut().unwrap();
    let card = combat.hand[0].clone();
    combat.hand.clear();
    combat.discard.clear();
    combat.exhaust.clear();
    combat.played_powers.clear();
    combat.stasis.clear();
    combat.draw_count = 2;
    combat.known_draw = vec![
        combat::KnownDrawCardView { card: card.clone(), from_top: true, offset: 0 },
        combat::KnownDrawCardView { card, from_top: false, offset: 0 },
    ];
    assert_eq!(
        encode_run_m10(&decision, None, &RunCapacities::m10_default()),
        Err(RunSessionError::Input("inconsistent public known-draw card positions")),
    );
}

#[test]
fn m10_encoder_rejects_different_identities_claiming_the_same_draw_position() {
    // Synthetic conflicting facts: top and bottom point to the same sole card position.
    let mut decision = m10_combat_decision(EncounterId::CorruptHeart);
    let combat = decision.combat.as_mut().unwrap();
    let card = combat.hand[0].clone();
    let mut other = card.clone();
    other.instance = combat::CardInstanceId(other.instance.0 + 1000);
    combat.hand.clear();
    combat.discard.clear();
    combat.exhaust.clear();
    combat.played_powers.clear();
    combat.stasis.clear();
    combat.draw_count = 1;
    combat.known_draw = vec![
        combat::KnownDrawCardView { card, from_top: true, offset: 0 },
        combat::KnownDrawCardView { card: other, from_top: false, offset: 0 },
    ];
    assert_eq!(
        encode_run_m10(&decision, None, &RunCapacities::m10_default()),
        Err(RunSessionError::Input("inconsistent public known-draw card positions")),
    );
}

#[test]
fn m10_encoder_rejects_conflicting_views_of_one_known_draw_identity() {
    // Synthetic conflicting facts: both endpoints name one card, but disagree on upgrades.
    let mut decision = m10_combat_decision(EncounterId::CorruptHeart);
    let combat = decision.combat.as_mut().unwrap();
    let card = combat.hand[0].clone();
    let mut conflicting = card.clone();
    conflicting.upgrades += 1;
    combat.hand.clear();
    combat.discard.clear();
    combat.exhaust.clear();
    combat.played_powers.clear();
    combat.stasis.clear();
    combat.draw_count = 1;
    combat.known_draw = vec![
        combat::KnownDrawCardView { card, from_top: true, offset: 0 },
        combat::KnownDrawCardView { card: conflicting, from_top: false, offset: 0 },
    ];
    assert_eq!(
        encode_run_m10(&decision, None, &RunCapacities::m10_default()),
        Err(RunSessionError::Input("conflicting public known-draw card view")),
    );
}

#[test]
fn m10_encoder_maps_an_actual_copy_selection_and_its_playing_parent() {
    let decision=m10_dual_wield_selection_decision();
    let observation=encode_run_m10(&decision,None,&RunCapacities::m10_default()).unwrap();
    let cards=group(&observation,"combat_cards");
    let width=cards.shape[1];
    let playing=cards.values.chunks(width).position(|row|row[0]==9).unwrap();
    let card_vocab=m10_run_observation_spec(&RunCapacities::m10_default()).unwrap().descriptor();
    let dual_wield=card_vocab["vocabularies"]["card"].as_array().unwrap().iter().position(|value|value=="dual_wield").unwrap() as i64;
    assert_eq!(cards.values[playing*width+2],dual_wield);
    assert_eq!(cards.known[playing*width+12],1,"X/fixed/unplayable cost classification is a known field");
    let actions=group(&observation,"actions");
    assert!(!actions.values.chunks(8).any(|row|row[0]==11),"fixed-size Dual Wield does not encode an unoffered confirmation");
    let counts = group(&observation, "selection_counts");
    assert_eq!(&counts.values[..6], &[1, 0, 1, 1, 0, 9], "actual Dual Wield exposes its fixed selector, no confirmation, and copy-to-hand-2 purpose");
    assert_eq!(counts.known[..6], [1, 1, 1, 1, 1, 1]);
}

#[test]
fn m10_encoder_maps_an_actually_offered_elixir_confirmation() {
    let decision = m10_elixir_selection_decision();
    let selection = decision.combat.as_ref().unwrap().selection.as_ref().unwrap();
    assert_eq!(selection.operation, SelectionOperation::Move(combat::SelectionDisposition::Exhaust));
    let observation = encode_run_m10(&decision, None, &RunCapacities::m10_default()).unwrap();
    let actions = group(&observation, "actions");
    let confirm_row = decision.candidates.iter().position(|candidate|
        candidate.action == GameAction::Combat(CombatAction::ConfirmSelection)).unwrap();
    assert_eq!(actions.values[confirm_row * 8], 11, "the native menu's actual ConfirmSelection maps to appended verb 11");
    assert_eq!(observation.action_mask[confirm_row], 1);
    let counts = group(&observation, "selection_counts");
    assert_eq!(counts.values[0], selection.required as i64);
    assert_eq!(counts.values[1], 0);
    assert_eq!(counts.values[2], selection.minimum as i64);
    assert_eq!(counts.values[3], selection.maximum as i64);
    assert_eq!(counts.values[4], 1, "confirmation availability comes from a native legal action");
    assert_eq!(counts.values[5], 3, "Elixir moves selected hand cards to exhaust");
    assert_eq!(counts.known[..6], [1, 1, 1, 1, 1, 1]);
}

#[test]
fn m10_card_cost_modes_and_temporary_values_do_not_collapse_into_upgrade_count() {
    let mut decision=m10_combat_decision(EncounterId::CorruptHeart);
    let card=&mut decision.combat.as_mut().unwrap().hand[0];
    // Deliberately synthetic supplied-transport values; keep the permanent wrapper row aligned.
    card.id=CardId::Whirlwind;
    card.upgrades=0;
    card.effective_cost=None;
    card.values=Some(combat::CardPublicValues{base_damage:Some(5),base_block:None,magic_number:Some(3),damage_bonus:Some(2),
        cost_kind:combat::CardCostKind::X,free_play:true,exhaust_on_finish:false,ethereal:false,innate:false,temporary_cost:1});
    let origin=card.origin.clone();
    if let combat::CombatCardRef::Permanent(instance)=origin {
        let object=decision.objects.iter_mut().find(|object|object.instance==instance.0).unwrap();
        object.id=card.id;
        object.upgrades=card.upgrades;
    }
    let observation=encode_run_m10(&decision,None,&RunCapacities::m10_default()).unwrap();
    let cards=group(&observation,"combat_cards");
    let width=cards.shape[1];
    let row=cards.values.chunks(width).position(|values|values[0]==1&&values[2]!=0).unwrap();
    assert_eq!(cards.values[row*width+3],0,"permanent upgrades are kept separate from public combat values");
    assert_eq!(&cards.values[row*width+8..row*width+18],&[5,0,3,2,2,1,0,0,0,1]);
    assert_eq!(&cards.known[row*width+8..row*width+18],&[1,0,1,1,1,1,1,1,1,1]);
}

#[test]
fn m10_act4_public_fields_are_known_only_at_qualified_boundaries() {
    let heart = m10_combat_decision(EncounterId::CorruptHeart);
    let heart_obs = encode_run_m10(&heart, None, &RunCapacities::m10_default()).unwrap();
    let state = group(&heart_obs, "combat_state");
    assert_eq!(&state.values[..4], &[0, 0, 300, 300]);
    assert_eq!(state.known[..4], [0, 0, 1, 1]);
    let heart_enemy = group(&heart_obs, "enemies");
    assert_eq!(heart_enemy.values[11], 0, "initial Debilitate is not an attack hit-count");
    assert_eq!(heart_enemy.known[11], 0);

    let pair = m10_combat_decision(EncounterId::ShieldAndSpear);
    let pair_combat=pair.combat.as_ref().unwrap();
    assert_eq!(pair_combat.enemies.iter().map(|enemy|enemy.intent.as_str()).collect::<Vec<_>>(),["shield_fortify","spear_burn_strike"]);
    let pair_obs = encode_run_m10(&pair, None, &RunCapacities::m10_default()).unwrap();
    let pair_state = group(&pair_obs, "combat_state");
    assert_eq!(&pair_state.values[..2], &[1, 1]);
    assert_eq!(pair_state.known[..2], [1, 1]);
    let enemies = group(&pair_obs, "enemies");
    let intent_vocab=m10_run_observation_spec(&RunCapacities::m10_default()).unwrap().descriptor();
    let intent_id=|name:&str|intent_vocab["vocabularies"]["intent"].as_array().unwrap().iter().position(|value|value==name).unwrap() as i64;
    assert_eq!(enemies.values[10], 1, "Shield is explicitly left");
    assert_eq!(enemies.values[4], intent_id("shield_fortify"), "the actual published Shield intent is encoded");
    assert_eq!(enemies.values[11], 0, "Fortify is not an attack with a known hit count");
    assert_eq!(enemies.values[22], 2, "Spear is explicitly right");
    assert_eq!(enemies.values[16], intent_id("spear_burn_strike"), "the actual published Spear intent is encoded");
    assert_eq!(enemies.values[23], 2, "Spear Burn Strike is two hits");
    assert_eq!(enemies.known[10], 1);
    assert_eq!(enemies.known[4], 1);
    assert_eq!(enemies.known[11], 0, "unknown intent hit count is represented by known0");
    assert_eq!(enemies.known[22], 1);
    assert_eq!(enemies.known[16], 1);
    assert_eq!(enemies.known[23], 1);
    assert_eq!(pair_state.known[2], 0, "Heart-only allowance remains unknown in a pair fight");
}

#[test]
fn m10_encoder_fails_closed_for_an_actual_native_retain_selection() {
    let mut decision = m10_decision();
    let cards = std::iter::once(combat::permanent_live_card(CardId::WellLaidPlans, combat::CardInstanceId(1), 0))
        .chain((2..=5).map(|instance| combat::permanent_live_card(CardId::Strike, combat::CardInstanceId(instance), 0)))
        .collect::<Vec<_>>();
    align_public_deck(&mut decision, &cards);
    let setup = combat::enemies::setup(
        EncounterId::CorruptHeart,
        4,
        0,
        &mut RunRng::new([102; 32], combat::rng::Partition::Test, 0),
        combat::enemies::SetupContext {
            player: combat::PlayerState { hp: 80, max_hp: 80, block: 0, energy: 3, max_energy: 3, draw_per_turn: 5, powers: vec![] },
            player_powers: vec![],
            character: Character::Ironclad,
            cards,
            opening_draw: 5,
            encounter_visit: 0,
        },
    ).unwrap();
    let mut items = Some(CombatItemContext::new(vec![RelicId::BurningBlood], vec![None, None, None]));
    let mut native = combat::Combat::enter_with_item_context(setup, &mut items).unwrap();
    let well_laid_plans = native.public_full().unwrap().hand.iter()
        .find(|card| card.id == CardId::WellLaidPlans).unwrap().instance.0;
    native.step_full(&CombatAction::PlayCard { instance: well_laid_plans, target: None }).unwrap();
    assert!(native.public_full().unwrap().player.powers.iter()
        .any(|power| power.id.0 == "retain_cards" && power.amount == 1));
    native.step_full(&CombatAction::EndTurn).unwrap();
    let legal = native.legal_actions_full().unwrap();
    assert!(legal.contains(&CombatAction::ConfirmSelection));
    let public = native.public_full().unwrap();
    let selection = public.selection.as_ref().expect("the native engine owns a live retain selection");
    assert_eq!(selection.parent, combat::SelectionParent::EndTurn { turn: public.turn });
    assert_eq!(selection.operation, SelectionOperation::Retain);

    decision.combat = Some(public);
    decision.room = None;
    decision.offers.clear();
    decision.candidates = native_combat_candidates(legal, 1400);
    decision.selection = None;
    assert_eq!(
        encode_run_m10(&decision, None, &RunCapacities::m10_default()),
        Err(RunSessionError::UnsupportedBoundary("M10 v2 cannot encode M11 retain selections")),
        "the frozen M10 encoder rejects the native M11 selector before enum encoding",
    );
}

#[test]
fn m10_encoder_fails_closed_for_an_actual_nightmare_capture_selection() {
    let mut decision = m10_decision();
    let cards = std::iter::once(combat::permanent_live_card(CardId::Nightmare, combat::CardInstanceId(1), 0))
        .chain((2..=5).map(|instance| combat::permanent_live_card(CardId::Strike, combat::CardInstanceId(instance), 0)))
        .collect::<Vec<_>>();
    align_public_deck(&mut decision, &cards);
    let setup = combat::enemies::setup(
        EncounterId::CorruptHeart,
        4,
        0,
        &mut RunRng::new([103; 32], combat::rng::Partition::Test, 0),
        combat::enemies::SetupContext {
            player: combat::PlayerState { hp: 80, max_hp: 80, block: 0, energy: 3, max_energy: 3, draw_per_turn: 5, powers: vec![] },
            player_powers: vec![],
            character: Character::Ironclad,
            cards,
            opening_draw: 5,
            encounter_visit: 0,
        },
    ).unwrap();
    let mut items = Some(CombatItemContext::new(vec![RelicId::BurningBlood], vec![None, None, None]));
    let mut native = combat::Combat::enter_with_item_context(setup, &mut items).unwrap();
    let nightmare = native.public_full().unwrap().hand.iter()
        .find(|card| card.id == CardId::Nightmare).unwrap().instance.0;
    native.step_full(&CombatAction::PlayCard { instance: nightmare, target: None }).unwrap();
    let public = native.public_full().unwrap();
    let selection = public.selection.as_ref().expect("played Nightmare owns its real remaining-hand selector");
    assert_eq!(selection.parent, combat::SelectionParent::Card(
        combat::CardHandle::parse(&nightmare.to_string()).unwrap()));
    assert_eq!(selection.operation, SelectionOperation::NightmareCapture);

    let legal = native.legal_actions_full().unwrap();
    decision.combat = Some(public);
    decision.room = None;
    decision.offers.clear();
    decision.candidates = native_combat_candidates(legal, 1500);
    decision.selection = None;
    assert_eq!(
        encode_run_m10(&decision, None, &RunCapacities::m10_default()),
        Err(RunSessionError::Input("unsupported CardId")),
        "the actual M11 card remains outside the frozen M10 card vocabulary",
    );

    // Codec-only negative control: mutate a supported native selection view to
    // the new operation so the frozen M10 operation boundary is exercised
    // independently of the earlier unknown-card rejection above.
    let mut codec_fixture = m10_dual_wield_selection_decision();
    codec_fixture.combat.as_mut().unwrap().selection.as_mut().unwrap().operation =
        SelectionOperation::NightmareCapture;
    assert_eq!(
        encode_run_m10(&codec_fixture, None, &RunCapacities::m10_default()),
        Err(RunSessionError::UnsupportedBoundary("M10 v2 cannot encode M11 Nightmare selections")),
        "M10 v2 rejects the unsupported selector operation without extending its descriptor",
    );
}
